# Changelog

Every change of `xx` is written here under **Unreleased** as it lands; a release moves those
lines under its version, and the section becomes the notes of the GitHub release. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `xx release publish <TAG> --identity <VAULT>/<IDENTITY> [--draft]`: the whole release in one
  command. GitHub Actions builds the draft, the identity signs its archive (a locked vault opens
  for this command only), the signature is checked as `xx update` checks it, and the release is
  published.
- `xx wallet`: the methods of Mirrornet's `cli_wallet`, under their own names. First
  `get_global_properties` and `get_dynamic_global_properties`, the latter with `--watch`
  (a line per block until Ctrl-C); `-o json` prints the node's object, `--network` picks the
  network (the built-in `mirrorboards` by default).
- `xx wallet import_balance <ACCOUNT> --identity <VAULT>/<IDENTITY> [--preview]`: claims the
  balances genesis put under an identity's keys into an account, in one transaction signed by the
  vault's session; `--preview` shows the claim and sends nothing.
- `xx wallet`: every read of `cli_wallet` (61 commands): the chain (`info`, `about`, `get_object`,
  `get_block`), accounts and their balances, history, vesting and storage, assets, the market,
  witnesses and the committee, HTLCs, transactions (`get_transaction_id`, `serialize_transaction`,
  `get_transaction_signers`), signed messages, and Mirrornet's data rooms, content cards and fee
  grants. A node's refusal prints its message, not its whole stack.
- `xx wallet` writes, signed by a vault's identity with `--preview` in place of `broadcast false`:
  `transfer`, `transfer2` (memos sealed by the vault), `register_account` (keys as `MBT…` or
  `<VAULT>/<IDENTITY>`), `upgrade_account`, `account_store_map`; and `sign_memo`, `read_memo`,
  `sign_message`, which send nothing. A preview prints the operations field by field.
- Tab completion: `xx completion fish|zsh|bash`. The shell asks xx at every Tab, so it also
  completes vault names, identities as `<VAULT>/<IDENTITY>` and networks. `install.sh` sets it up
  for fish (`~/.config/fish/completions/xx.fish`) and zsh (a line in `~/.zshrc`).

### Removed

- `xx release sign`, replaced by `xx release publish`.

## [0.2.0] - 2026-11-25

### Added

- `xx vault guard local add|remove <NAME> yubikey [--key <ID>]`: a YubiKey that unlocks the vault
  on this computer only; it stays out of the vault's file, its server copy and its backups.
- `xx vault create --local-guard yubikey`: the plugged-in key becomes the local guard, proven by
  the new recovery code.
- `xx vault import --local-guard yubikey --with recovery-code`: the copy opens with its recovery
  code, and the plugged-in key becomes the local guard.
- `xx release sign <FILE> --version <TAG> --identity <VAULT>/<IDENTITY>`: a release archive
  signed by an XKeys identity through its vault session, as `<FILE>.sig`.
- `xx update` installs only releases signed by a key built into xx: from this release on,
  `xx-releases/releaser` (`0226c910…9a25a370`). Updating from 0.1.0, which trusts no key yet,
  checks the sha256 alone.
- Releases are built in GitHub Actions as drafts, which the owner signs and publishes.

### Fixed

- A vault that opens only with YubiKeys waits for one instead of asking a paper code it has not
  (in `x` too).

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

[Unreleased]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mirrorboards-cli/cli-xx/releases/tag/v0.1.0
