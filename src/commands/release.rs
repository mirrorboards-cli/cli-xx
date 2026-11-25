//! `xx release sign <FILE> --version <TAG> --identity <VAULT>/<IDENTITY>`: the owner signs a
//! release archive before it goes to GitHub (`crate::release_signature`). The identity's vault
//! must be unlocked; its session signs, and `<FILE>.sig` is written beside the archive.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use sha2::{Digest, Sha256};

use crate::release_signature::{ReleaseSignature, digest};

#[derive(Subcommand)]
pub enum Command {
    /// Sign a release archive with an XKeys identity; writes <FILE>.sig
    Sign(SignArguments),
}

#[derive(Args)]
pub struct SignArguments {
    /// The release archive, e.g. xx-macos-arm64.tar.gz
    file: PathBuf,

    /// The release's tag, e.g. v0.2.0
    #[arg(long, value_name = "TAG")]
    version: String,

    /// The identity that signs releases; its vault must be unlocked (xx vault unlock <VAULT>)
    #[arg(long, value_name = "VAULT>/<IDENTITY")]
    identity: String,
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::Sign(arguments) => sign(arguments).await,
    }
}

async fn sign(arguments: SignArguments) -> Result<()> {
    let Some((vault, identity)) = arguments.identity.split_once('/') else {
        bail!("name the vault too: --identity <VAULT>/<IDENTITY>");
    };
    let archive_sha256 = sha256_of(&arguments.file)?;

    let home = xkeys_cli::vaults::Vaults::default_location()?.named(vault)?;
    let Some(mut session) = xkeys_cli::session::running_session(&home).await? else {
        bail!("{vault} is locked: xx vault unlock {vault}");
    };
    let description = session.describe_identity(identity).await?;
    let signer = description
        .keys
        .iter()
        .find(|key| key.role == "active")
        .map(|key| key.key.clone())
        .with_context(|| format!("{vault}/{identity} has no active key"))?;

    let signed = digest(&arguments.version, &archive_sha256);
    let signature = session.sign_digest(identity, "active", &signed).await?;
    let file = ReleaseSignature::new(&arguments.version, &archive_sha256, &signer, &signature);

    let mut path = arguments.file.clone().into_os_string();
    path.push(".sig");
    let path = PathBuf::from(path);
    let mut text = serde_json::to_string_pretty(&file).context("the signature is no JSON")?;
    text.push('\n');
    std::fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))?;

    println!(
        "{}: signed {} as {vault}/{identity} (key {signer})",
        path.display(),
        arguments.version
    );

    Ok(())
}

fn sha256_of(file: &Path) -> Result<String> {
    let bytes = std::fs::read(file).with_context(|| format!("cannot read {}", file.display()))?;

    Ok(hex::encode(Sha256::digest(&bytes)))
}
