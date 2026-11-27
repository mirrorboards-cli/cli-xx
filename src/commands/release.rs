//! `xx release publish <TAG> --identity <VAULT>/<IDENTITY> [--draft]`: the whole release of xx in
//! one command (owner, 2026-11-25). GitHub Actions builds the draft (`.github/workflows/release.yml`)
//! with an archive for each system xx runs on, this computer signs every archive with an XKeys
//! identity (`crate::release_signature`), checks each signature as `xx update` will, uploads them
//! and publishes the release. The signing key never leaves the vault; GitHub is reached through the
//! `gh` command line, signed in already.

use std::path::{Path, PathBuf};
use std::process::Command as Process;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::release_signature::{RELEASE_KEYS, ReleaseSignature, digest};

const REPOSITORY: &str = "mirrorboards-cli/cli-xx";
const WORKFLOW: &str = "release.yml";
/// The archives of a release, one for each system xx runs on, each signed on its own: the
/// signature binds the version and the archive's sha256 (`release_signature::digest`).
const ARCHIVES: [&str; 2] = ["xx-macos-arm64.tar.gz", "xx-linux-x64.tar.gz"];
/// How often the build is asked about while it runs.
const POLL: Duration = Duration::from_secs(20);

#[derive(Subcommand)]
pub enum Command {
    /// Build, sign and publish a release of xx: GitHub Actions builds, your identity signs
    Publish(PublishArguments),
}

#[derive(Args)]
pub struct PublishArguments {
    /// The release's tag, e.g. v0.3.0; cli-xx/Cargo.toml on main must carry that version
    tag: String,

    /// The identity that signs releases, e.g. xx-releases/releaser; a locked vault opens for this
    /// command only
    #[arg(long, value_name = "VAULT>/<IDENTITY")]
    identity: String,

    /// Sign and upload the signature, but leave the release a draft to look at first
    #[arg(long)]
    draft: bool,
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::Publish(arguments) => publish(arguments).await,
    }
}

async fn publish(arguments: PublishArguments) -> Result<()> {
    let tag = &arguments.tag;
    let Some((vault, identity)) = arguments.identity.split_once('/') else {
        bail!("name the vault too: --identity <VAULT>/<IDENTITY>");
    };

    let state = release_state(tag)?;
    match state {
        ReleaseState::Published => {
            bail!("{tag} is published already");
        }
        ReleaseState::Draft { missing } if missing.is_empty() => {
            eprintln!("{tag}: the draft has its archives; signing them");
        }
        ReleaseState::Draft { missing } => {
            bail!(
                "the draft {tag} has no {}; delete it (gh release delete {tag} -R {REPOSITORY}) and run again",
                missing.join(", ")
            );
        }
        ReleaseState::Missing => {
            build_draft(tag)?;
        }
    }

    let directory = download_archives(tag)?;
    let mut archive_sha256s = Vec::new();
    for archive in ARCHIVES {
        let archive_sha256 = checked_sha256(&directory.join(archive))?;
        archive_sha256s.push(archive_sha256);
    }

    // One session signs every archive, so the vault opens once.
    let (signer, signatures) =
        xkeys_cli::xx::one_command_session::with_vault_session(vault, async |session| {
            let description = session.describe_identity(identity).await?;
            let signer = description
                .keys
                .iter()
                .find(|key| key.role == "active")
                .map(|key| key.key.clone())
                .with_context(|| format!("{vault}/{identity} has no active key"))?;

            let mut signatures = Vec::new();
            for archive_sha256 in &archive_sha256s {
                let signed = digest(tag, archive_sha256);
                let signature = session.sign_digest(identity, "active", &signed).await?;
                signatures.push(signature);
            }

            Ok((signer, signatures))
        })
        .await?;

    // Every signature is checked before any is uploaded.
    let mut signature_paths = Vec::new();
    for ((archive, archive_sha256), signature) in
        ARCHIVES.iter().zip(&archive_sha256s).zip(&signatures)
    {
        let file = ReleaseSignature::new(tag, archive_sha256, &signer, signature);
        file.check(tag, archive_sha256, RELEASE_KEYS)
            .with_context(|| {
                format!("xx update would refuse the signature of {archive}; nothing was uploaded")
            })?;

        let signature_path = directory.join(format!("{archive}.sig"));
        let mut text = serde_json::to_string_pretty(&file).context("the signature is no JSON")?;
        text.push('\n');
        std::fs::write(&signature_path, text)
            .with_context(|| format!("cannot write {}", signature_path.display()))?;
        signature_paths.push(signature_path);
    }

    let mut upload = vec!["release", "upload", tag, "-R", REPOSITORY, "--clobber"];
    for signature_path in &signature_paths {
        upload.push(path_text(signature_path)?);
    }
    gh(&upload)?;
    eprintln!(
        "{tag}: {} signed as {vault}/{identity} (key {signer})",
        ARCHIVES.join(" and ")
    );

    if arguments.draft {
        println!("{tag} stays a draft: gh release edit {tag} -R {REPOSITORY} --draft=false");

        return Ok(());
    }

    let url = gh(&["release", "edit", tag, "-R", REPOSITORY, "--draft=false"])?;
    println!("{}", url.trim());

    Ok(())
}

enum ReleaseState {
    Missing,
    /// A draft, with the archives it lacks.
    Draft {
        missing: Vec<&'static str>,
    },
    Published,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseView {
    is_draft: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
}

fn release_state(tag: &str) -> Result<ReleaseState> {
    let output = Process::new("gh")
        .args([
            "release",
            "view",
            tag,
            "-R",
            REPOSITORY,
            "--json",
            "isDraft,assets",
        ])
        .output()
        .context("cannot run gh, the GitHub command line")?;

    let not_found = String::from_utf8_lossy(&output.stderr).contains("release not found");
    if not_found {
        return Ok(ReleaseState::Missing);
    }
    if !output.status.success() {
        bail!(
            "gh release view {tag}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let view: ReleaseView =
        serde_json::from_slice(&output.stdout).context("gh release view gave no release")?;
    if !view.is_draft {
        return Ok(ReleaseState::Published);
    }

    let missing = missing_archives(&view.assets);

    Ok(ReleaseState::Draft { missing })
}

/// The archives of [`ARCHIVES`] that `assets` lack.
fn missing_archives(assets: &[Asset]) -> Vec<&'static str> {
    ARCHIVES
        .into_iter()
        .filter(|archive| !assets.iter().any(|asset| asset.name == *archive))
        .collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Run {
    database_id: u64,
    status: String,
    conclusion: String,
    url: String,
}

/// Starts the workflow Release for `tag` and waits until it has drafted the release.
fn build_draft(tag: &str) -> Result<()> {
    let known_runs: Vec<u64> = workflow_runs()?.iter().map(|run| run.database_id).collect();

    gh(&[
        "workflow",
        "run",
        WORKFLOW,
        "-R",
        REPOSITORY,
        "-f",
        &format!("version={tag}"),
    ])?;

    let started = Instant::now();
    let run = loop {
        std::thread::sleep(Duration::from_secs(3));

        let new_run = workflow_runs()?
            .into_iter()
            .find(|run| !known_runs.contains(&run.database_id));
        if let Some(run) = new_run {
            break run;
        }
        if started.elapsed() > Duration::from_secs(60) {
            bail!("the workflow {WORKFLOW} did not start within a minute");
        }
    };
    eprintln!(
        "{tag}: building in GitHub Actions, about 15 minutes ({})",
        run.url
    );

    loop {
        std::thread::sleep(POLL);

        let output = gh(&[
            "run",
            "view",
            &run.database_id.to_string(),
            "-R",
            REPOSITORY,
            "--json",
            "databaseId,status,conclusion,url",
        ])?;
        let now: Run = serde_json::from_str(&output).context("gh run view gave no run")?;
        if now.status != "completed" {
            continue;
        }
        if now.conclusion != "success" {
            bail!("the build ended with {}: {}", now.conclusion, now.url);
        }

        let minutes = started.elapsed().as_secs() / 60;
        eprintln!("{tag}: built in {minutes} min");

        return Ok(());
    }
}

fn workflow_runs() -> Result<Vec<Run>> {
    let output = gh(&[
        "run",
        "list",
        "-R",
        REPOSITORY,
        "--workflow",
        WORKFLOW,
        "-L",
        "10",
        "--json",
        "databaseId,status,conclusion,url",
    ])?;

    serde_json::from_str(&output).context("gh run list gave no runs")
}

/// The draft's archives and their .sha256 files, in a directory of their own.
fn download_archives(tag: &str) -> Result<PathBuf> {
    let directory = std::env::temp_dir().join(format!("xx-release-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .with_context(|| format!("cannot make {}", directory.display()))?;

    let patterns: Vec<String> = ARCHIVES
        .iter()
        .flat_map(|archive| [archive.to_string(), format!("{archive}.sha256")])
        .collect();
    let mut download = vec![
        "release",
        "download",
        tag,
        "-R",
        REPOSITORY,
        "--clobber",
        "-D",
        path_text(&directory)?,
    ];
    for pattern in &patterns {
        download.push("-p");
        download.push(pattern);
    }
    gh(&download)?;

    Ok(directory)
}

/// The archive's sha256, checked against the .sha256 the build wrote beside it.
fn checked_sha256(archive: &Path) -> Result<String> {
    let bytes =
        std::fs::read(archive).with_context(|| format!("cannot read {}", archive.display()))?;
    let actual = hex::encode(Sha256::digest(&bytes));

    let sha256_file = archive.with_extension("gz.sha256");
    let written = std::fs::read_to_string(&sha256_file)
        .with_context(|| format!("cannot read {}", sha256_file.display()))?;
    let expected = written.split_whitespace().next().unwrap_or_default();
    if expected != actual {
        bail!("the archive's sha256 is {actual}, its .sha256 says {expected}");
    }

    Ok(actual)
}

/// Runs `gh` and returns what it printed; its error output becomes the error.
fn gh(arguments: &[&str]) -> Result<String> {
    let output = Process::new("gh")
        .args(arguments)
        .output()
        .context("cannot run gh, the GitHub command line")?;
    if !output.status.success() {
        bail!(
            "gh {}: {}",
            arguments.first().copied().unwrap_or_default(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn path_text(path: &Path) -> Result<&str> {
    path.to_str()
        .with_context(|| format!("{} is not UTF-8", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str) -> Asset {
        Asset {
            name: name.to_owned(),
        }
    }

    #[test]
    fn a_draft_lacks_every_archive_it_does_not_carry() {
        let only_macos = [
            asset("xx-macos-arm64.tar.gz"),
            asset("xx-macos-arm64.tar.gz.sha256"),
        ];
        let both = [asset("xx-macos-arm64.tar.gz"), asset("xx-linux-x64.tar.gz")];

        assert_eq!(missing_archives(&only_macos), vec!["xx-linux-x64.tar.gz"]);
        assert!(missing_archives(&both).is_empty());
        assert_eq!(missing_archives(&[]), ARCHIVES.to_vec());
    }
}
