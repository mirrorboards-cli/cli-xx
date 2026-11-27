//! `xx update`: this program replaced by its newest release on GitHub, as the owner accepted it on
//! 2026-11-25 (`cli-spec/xx.typ`). Releases live in the public repository mirrorboards-cli/cli-xx,
//! tagged `v<version>`, each with `xx-<os>-<arch>.tar.gz` and its `.sha256`.
//!
//! The archive is checked against its sha256 before anything changes, and this program is swapped
//! by renaming the new one into its place, so a failure leaves the old one whole. The checksum
//! guards against a broken download, not against someone who takes over the repository; signing
//! the releases is a later step.

use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use self_update::backends::github::ReleaseList;
use self_update::update::{Release, ReleaseAsset};
use self_update::{ArchiveKind, Compression, Download, Extract};
use serde::Serialize;
use sha2::{Digest, Sha256};
use xkeys_cli::xx::vault::lock::{Locked, lock_every_unlocked_vault};

use crate::release_signature::{RELEASE_KEYS, ReleaseSignature};

const REPOSITORY_OWNER: &str = "mirrorboards-cli";
const REPOSITORY_NAME: &str = "cli-xx";
const PROGRAM: &str = "xx";

/// Pulumi's Rust language host, which a release carries beside xx and `xx mirrorboard deploy` puts
/// first in Pulumi's PATH (PT-4 of cli-spec xx.typ). Releases before 0.5.0 carry xx alone.
const LANGUAGE_HOST: &str = "pulumi-language-rust";

#[derive(Args)]
pub struct Arguments {
    /// Only say whether a newer release is there; change nothing
    #[arg(long, conflicts_with = "version")]
    check: bool,

    /// Install this release instead of the newest, e.g. v0.2.0
    #[arg(long, value_name = "TAG")]
    version: Option<String>,

    /// text or json
    #[arg(short = 'o', long, value_name = "FORMAT", value_enum, default_value_t = Format::Text)]
    output: Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Text,
    Json,
}

/// What `xx update` came to.
#[derive(Debug, Serialize)]
struct Updated {
    /// The version this program had.
    from: String,
    /// The release chosen.
    to: String,
    /// Whether this program was replaced now.
    replaced: bool,
    /// The vaults whose sessions were locked because they run the program that was replaced.
    locked_vaults: Vec<String>,
}

pub fn run(arguments: Arguments) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let asset_name = asset_name()?;

    let releases = ReleaseList::configure()
        .repo_owner(REPOSITORY_OWNER)
        .repo_name(REPOSITORY_NAME)
        .build()
        .context("cannot ask GitHub for the releases of xx")?
        .fetch()
        .context("cannot ask GitHub for the releases of xx")?;
    let release = chosen_release(&releases, arguments.version.as_deref())?;
    let target = release.version.trim_start_matches('v').to_owned();

    let newer = self_update::version::bump_is_greater(current, &target).unwrap_or(false);
    let asked_for_this_one = arguments.version.is_some() && target != current;
    let replaces = !arguments.check && (newer || asked_for_this_one);

    let mut locked_sessions = Vec::new();
    if replaces {
        install(release, &asset_name)?;
        locked_sessions = lock_sessions_of_the_replaced_program();
    }

    let updated = Updated {
        from: current.to_owned(),
        to: target,
        replaced: replaces,
        locked_vaults: locked_sessions
            .iter()
            .map(|locked| locked.vault.clone())
            .collect(),
    };
    match arguments.output {
        Format::Json => {
            let text = serde_json::to_string_pretty(&updated).context("the result is no JSON")?;
            println!("{text}");
        }
        Format::Text => {
            println!("{}", line_of(&updated, newer, arguments.check));
            for locked in &locked_sessions {
                println!("{}", locked_line(locked));
            }
        }
    }

    Ok(())
}

/// Every unlocked vault's session still runs the program that was just replaced, and would
/// refuse what the new one asks of it; locking them lets the next command open them again with
/// the new program. A failure does not undo the update: it is said, and the person locks the
/// vaults by hand.
fn lock_sessions_of_the_replaced_program() -> Vec<Locked> {
    let locked = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("cannot start the async runtime")
        .and_then(|runtime| runtime.block_on(lock_every_unlocked_vault()));

    match locked {
        Ok(locked) => locked,
        Err(error) => {
            eprintln!(
                "xx was updated, but not every session was locked ({error:#}); \
                 xx vault lock --all locks them, as they run the old xx"
            );

            Vec::new()
        }
    }
}

/// `locked mirrornet-mainnet-accounts: its session ran xx 0.3.0`
fn locked_line(locked: &Locked) -> String {
    let ran = locked.session_ran.as_deref().unwrap_or("an unknown xx");

    format!("locked {}: its session ran {ran}", locked.vault)
}

/// `xx-macos-arm64.tar.gz`: the archive of this system and processor.
fn asset_name() -> Result<String> {
    let os = match std::env::consts::OS {
        "macos" => "macos",
        "linux" => "linux",
        other => bail!("xx has no releases for {other}"),
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        other => bail!("xx has no releases for the processor {other}"),
    };

    Ok(format!("{PROGRAM}-{os}-{arch}.tar.gz"))
}

/// The release tagged `tag`, else the newest one.
fn chosen_release<'a>(releases: &'a [Release], tag: Option<&str>) -> Result<&'a Release> {
    let Some(tag) = tag else {
        return releases
            .first()
            .context("mirrorboards-cli/cli-xx has no releases yet");
    };

    let wanted = tag.trim_start_matches('v');
    releases
        .iter()
        .find(|release| release.version.trim_start_matches('v') == wanted)
        .with_context(|| format!("mirrorboards-cli/cli-xx has no release {tag}"))
}

/// Downloads the archive and its checksum, checks it, and swaps this program for the one in it.
fn install(release: &Release, asset_name: &str) -> Result<()> {
    let archive_asset = asset_of(release, asset_name)?;
    let checksum_asset = asset_of(release, &format!("{asset_name}.sha256"))?;

    let work = tempfile_dir()?;
    let archive = work.path().join(asset_name);
    eprintln!("downloading {asset_name} of {} …", release.version);
    download(archive_asset, &archive)?;

    let mut expected = Vec::new();
    Download::from_url(&checksum_asset.download_url)
        .set_header(
            reqwest_accept(),
            "application/octet-stream"
                .parse()
                .expect("a valid header value"),
        )
        .download_to(&mut expected)
        .context("cannot download the checksum")?;
    let archive_sha256 = check_sha256(&archive, &expected)?;
    eprintln!("sha256 ok");
    check_signature(release, asset_name, &archive_sha256)?;

    Extract::from_source(&archive)
        .archive(ArchiveKind::Tar(Some(Compression::Gz)))
        .extract_file(work.path(), PROGRAM)
        .context("the archive holds no xx")?;
    let new_program = work.path().join(PROGRAM);

    self_update::self_replace::self_replace(&new_program).map_err(|error| {
        let denied = error.kind() == std::io::ErrorKind::PermissionDenied;
        match denied {
            true => anyhow::anyhow!(
                "this program cannot be replaced where it is, without rights there: sudo xx update"
            ),
            false => anyhow::anyhow!("cannot replace this program: {error}"),
        }
    })?;

    let carries_language_host = Extract::from_source(&archive)
        .archive(ArchiveKind::Tar(Some(Compression::Gz)))
        .extract_file(work.path(), LANGUAGE_HOST)
        .is_ok();
    if carries_language_host {
        let program = std::env::current_exe().context("the path of this program is unknown")?;
        put_beside(&work.path().join(LANGUAGE_HOST), &program)
            .with_context(|| format!("xx was updated, but {LANGUAGE_HOST} beside it was not"))?;
    }

    Ok(())
}

/// Puts `file` beside `program` under its own name. It is copied in under another name and
/// renamed, so a failure leaves the one there whole.
fn put_beside(file: &Path, program: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let name = file
        .file_name()
        .with_context(|| format!("{} has no file name", file.display()))?
        .to_string_lossy()
        .into_owned();
    let destination = program.with_file_name(&name);
    let incoming = program.with_file_name(format!(".{name}.incoming"));

    std::fs::copy(file, &incoming).map_err(|error| {
        let denied = error.kind() == std::io::ErrorKind::PermissionDenied;
        match denied {
            true => anyhow::anyhow!(
                "{name} cannot be written beside this program, without rights there: sudo xx update"
            ),
            false => anyhow::anyhow!("cannot write {}: {error}", incoming.display()),
        }
    })?;
    std::fs::set_permissions(&incoming, std::fs::Permissions::from_mode(0o755))
        .with_context(|| format!("cannot make {} runnable", incoming.display()))?;
    std::fs::rename(&incoming, &destination)
        .with_context(|| format!("cannot put {name} in place of {}", destination.display()))?;

    Ok(())
}

fn asset_of<'a>(release: &'a Release, name: &str) -> Result<&'a ReleaseAsset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .with_context(|| format!("release {} has no {name}", release.version))
}

fn download(asset: &ReleaseAsset, to: &Path) -> Result<()> {
    let mut file =
        std::fs::File::create(to).with_context(|| format!("cannot write {}", to.display()))?;

    Download::from_url(&asset.download_url)
        .set_header(
            reqwest_accept(),
            "application/octet-stream"
                .parse()
                .expect("a valid header value"),
        )
        .download_to(&mut file)
        .with_context(|| format!("cannot download {}", asset.name))
}

/// The release's signature, when this build trusts a release key; without one the sha256 is all
/// that is checked, and that is said.
fn check_signature(release: &Release, asset_name: &str, archive_sha256: &str) -> Result<()> {
    if RELEASE_KEYS.is_empty() {
        eprintln!(
            "this build trusts no release key yet: the release is checked by its sha256 only"
        );
        return Ok(());
    }

    let signature_asset = asset_of(release, &format!("{asset_name}.sig"))
        .context("the release is not signed, and xx installs only signed releases")?;
    let mut text = Vec::new();
    Download::from_url(&signature_asset.download_url)
        .set_header(
            reqwest_accept(),
            "application/octet-stream"
                .parse()
                .expect("a valid header value"),
        )
        .download_to(&mut text)
        .context("cannot download the release's signature")?;
    let signature: ReleaseSignature =
        serde_json::from_slice(&text).context("the release's signature is no JSON")?;

    signature.check(&release.version, archive_sha256, RELEASE_KEYS)?;
    eprintln!("signature ok ({})", signature.signer);

    Ok(())
}

/// The archive's sha256 must be the first word of the checksum file; returns it.
fn check_sha256(archive: &Path, checksum_file: &[u8]) -> Result<String> {
    let text = String::from_utf8_lossy(checksum_file);
    let expected = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_lowercase();

    let mut bytes = Vec::new();
    std::fs::File::open(archive)
        .and_then(|mut file| file.read_to_end(&mut bytes))
        .with_context(|| format!("cannot read {}", archive.display()))?;
    let actual = hex::encode(Sha256::digest(&bytes));

    if actual != expected {
        bail!(
            "the download's sha256 is {actual}, but the release says {expected}; nothing was changed"
        );
    }

    Ok(actual)
}

fn line_of(updated: &Updated, newer: bool, only_checks: bool) -> String {
    let Updated {
        from, to, replaced, ..
    } = updated;

    match (replaced, newer, only_checks) {
        (true, _, _) => format!("xx: updated from {from} to {to}"),
        (false, true, true) => format!("xx {from}: {to} is out; xx update installs it"),
        (false, _, _) => format!("xx {from} is the newest release"),
    }
}

/// The `Accept` header name, of the HTTP client self_update uses.
fn reqwest_accept() -> reqwest::header::HeaderName {
    reqwest::header::ACCEPT
}

fn tempfile_dir() -> Result<self_update::TempDir> {
    self_update::TempDir::new().context("cannot make a temporary folder")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_checksum_must_match() {
        let directory = tempfile_dir().unwrap();
        let archive = directory.path().join("xx.tar.gz");
        std::fs::write(&archive, b"archive").unwrap();
        let right = format!("{}  xx.tar.gz\n", hex::encode(Sha256::digest(b"archive")));

        assert!(check_sha256(&archive, right.as_bytes()).is_ok());
        assert!(check_sha256(&archive, b"0000  xx.tar.gz").is_err());
    }

    #[test]
    fn a_locked_session_is_reported_with_the_xx_it_ran() {
        let locked = Locked {
            vault: "mirrornet-mainnet-accounts".to_owned(),
            was_unlocked: true,
            session_ran: Some("xx 0.3.0".to_owned()),
        };

        assert_eq!(
            locked_line(&locked),
            "locked mirrornet-mainnet-accounts: its session ran xx 0.3.0"
        );
    }

    #[test]
    fn the_language_host_goes_beside_the_program_in_place_of_the_old_one() {
        use std::os::unix::fs::PermissionsExt;

        let installed = tempfile_dir().unwrap();
        let unpacked = tempfile_dir().unwrap();
        let program = installed.path().join("xx");
        std::fs::write(&program, b"xx").unwrap();
        std::fs::write(installed.path().join(LANGUAGE_HOST), b"old host").unwrap();
        let new_host = unpacked.path().join(LANGUAGE_HOST);
        std::fs::write(&new_host, b"new host").unwrap();

        put_beside(&new_host, &program).unwrap();

        let beside = installed.path().join(LANGUAGE_HOST);
        assert_eq!(std::fs::read(&beside).unwrap(), b"new host");
        let mode = std::fs::metadata(&beside).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
        assert!(!installed.path().join(".pulumi-language-rust.incoming").exists());
    }

    #[test]
    fn this_system_names_its_archive() {
        let name = asset_name().unwrap();

        assert!(
            name.starts_with("xx-") && name.ends_with(".tar.gz"),
            "{name}"
        );
    }
}
