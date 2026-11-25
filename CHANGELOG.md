# Changelog

Every change of `xx` is written here under **Unreleased** as it lands; a release moves those
lines under its version, and the section becomes the notes of the GitHub release. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `xx vault guard local add|remove <NAME> yubikey [--key <ID>]`: a YubiKey that unlocks the vault
  on this computer only; it stays out of the vault's file, its server copy and its backups.
- `xx vault create --local-guard yubikey`: the plugged-in key becomes the local guard, proven by
  the new recovery code.
- `xx vault import --local-guard yubikey --with recovery-code`: the copy opens with its recovery
  code, and the plugged-in key becomes the local guard.

### Changed

- A new local password is asked as "Password of the local guard".
- A file that is no vault is named as such, without pointing to `x`.

## [0.1.0] - 2026-11-25

The first release, for macOS on Apple Silicon (macOS 15 or newer).

### Added

- `xx vault create <NAME>`, with `--wizard`: portable guards (`--yubikey`, a recovery code),
  a local guard (`--local-guard password`), `--sync` on an xkeys server or in a `.xkeys` file,
  and a one-time `--backup`.
- `xx vault list` (`-o wide|name|json|table`), `xx vault unlock` (`--for`, `--with`),
  `xx vault lock`, `xx vault sync` (`--to <FILE>`), `xx vault import <FILE> [NAME] [--sync]`,
  `xx vault remove`.
- `xx vault guard list|portable|local`: what opens a vault, on any computer or on this one.
- `xx identity create <VAULT>/<NAME>`.
- `xx stacks init|deploy|preview <PATH>`: Pulumi platforms, their passphrase opened by an XKeys
  identity, chosen by itself when one vault here has a slot.
- `xx update` and `install.sh`: install and update from the GitHub releases, checked by sha256.

[Unreleased]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mirrorboards-cli/cli-xx/releases/tag/v0.1.0
