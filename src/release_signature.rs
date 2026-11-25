//! The signature of a release of xx (owner, 2026-11-25: releases signed by an XKeys identity
//! rather than by a key in a file). The releaser's vault session signs, with the identity's
//! `active` key, a digest of a domain of its own, the version and the archive's sha256; `xx update`
//! checks it against the key built into this program. The domain keeps the signature from meaning
//! anything else, such as a transaction on a chain.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The key of the identity that signs releases, compressed secp256k1 in hex. `None` until the
/// owner names it: this build then checks releases by their sha256 alone.
pub const RELEASE_KEY: Option<&str> = None;

const DOMAIN: &str = "xx release v1";
const FORMAT: &str = "xx-release-signature-1";

/// The file `<archive>.sig` of a release.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSignature {
    pub format: String,
    /// The release's tag, e.g. `v0.2.0`.
    pub version: String,
    /// The archive's sha256, hex.
    pub sha256: String,
    /// The signer's public key, compressed secp256k1 in hex.
    pub signer: String,
    /// The compact signature (65 bytes), hex.
    pub signature: String,
}

/// What is signed: the domain, the version and the archive's sha256.
pub fn digest(version: &str, archive_sha256: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN.as_bytes());
    hasher.update([0]);
    hasher.update(version.as_bytes());
    hasher.update([0]);
    hasher.update(archive_sha256.as_bytes());

    hasher.finalize().into()
}

impl ReleaseSignature {
    pub fn new(version: &str, archive_sha256: &str, signer: &str, signature: &[u8; 65]) -> Self {
        Self {
            format: FORMAT.to_owned(),
            version: version.to_owned(),
            sha256: archive_sha256.to_owned(),
            signer: signer.to_owned(),
            signature: hex::encode(signature),
        }
    }

    /// Accepts the signature only when the trusted key made it, over this version and this
    /// archive.
    pub fn check(&self, version: &str, archive_sha256: &str, trusted_key: &str) -> Result<()> {
        if self.format != FORMAT {
            bail!(
                "the release's signature is of the format {}, xx reads {FORMAT}",
                self.format
            );
        }
        if self.version.trim_start_matches('v') != version.trim_start_matches('v') {
            bail!(
                "the signature is of release {}, not {version}",
                self.version
            );
        }
        if self.sha256 != archive_sha256 {
            bail!("the signature is of another archive than the one downloaded");
        }
        if self.signer != trusted_key {
            bail!(
                "the release is signed by {}, not by the key xx trusts",
                self.signer
            );
        }

        let key: [u8; 33] = hex::decode(trusted_key)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .context("the key xx trusts is no compressed public key")?;
        let signature = hex::decode(&self.signature).context("the signature is no hex")?;
        let signed = digest(&self.version, &self.sha256);
        if !xkeys_chain::sign::verify_compact(&signature, &signed, &key) {
            bail!("the release's signature does not hold; nothing was changed");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digest_binds_the_version_and_the_archive() {
        assert_ne!(digest("v0.2.0", "aa"), digest("v0.2.1", "aa"));
        assert_ne!(digest("v0.2.0", "aa"), digest("v0.2.0", "ab"));
    }

    #[test]
    fn another_signer_or_archive_is_refused() {
        let signature = ReleaseSignature::new("v0.2.0", "aa", "02ab", &[0; 65]);

        let other_signer = signature.check("v0.2.0", "aa", "03cd").unwrap_err();
        let other_archive = signature.check("v0.2.0", "bb", "02ab").unwrap_err();

        assert!(other_signer.to_string().contains("not by the key"));
        assert!(other_archive.to_string().contains("another archive"));
    }
}
