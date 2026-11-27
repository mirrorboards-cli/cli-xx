# Changelog

Every change of `xx` is written here under **Unreleased** as it lands; a release moves those
lines under its version, and the section becomes the notes of the GitHub release. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `xx mirrorboard deploy` names who decided: `signed by kacper-dela for acta-network` for a
  member who signs for a company alone (XSystem's new `decided_by_member`), `signed by ai-payer
  and ai-payee for ai-joint-company` for members who approve together, and `refused by ai-payee
  for ai-joint-company`; `-o json` gives `signed_by_members`.

### Fixed

- `xx mirrorboard deploy` takes a room or member its owner made by a proposal after the run that
  proposed it ended, also when the program changed the resource since, e.g. the room's
  description, and proposes the change; before, it proposed a second room.
  Its line says `✓ room notes: changed by its owner's proposal`.
- `xx mirrorboard sign` and `xx mirrorboard withdraw` end at once after Ctrl+C at their question;
  before, they waited for Enter. The end of input (Ctrl+D) ends the prompt's line too.
- `xx mirrorboard sign` forgets the journal entries of proposals that ended without its
  completion, such as one infra withdrew with `--approved` while it was approved.
- `xx mirrorboard deploy` names a change it took from a proposal its owner signed after the
  earlier run ended by the room or member, e.g. `✓ room dropzone: deleted by its owner's
  proposal`, not by the resource's name as if the run had made it.

## [0.5.1] - 2026-11-27

### Added

- `xx mirrorboard github [PATH] --identity <VAULT>/<IDENTITY> --registrar <URL>`: sets an
  instance up to deploy from GitHub Actions. It makes the CI vault (synced into a folder,
  `--sync=file://…`) and its account, paid by the XSystem's registrar; seals a slot of the stack's
  passphrase for CI, or makes a new stack with slots of CI and of `--identity`; sets the
  repository's secrets `INFRA_RECOVERY_CODE` (the recovery code, shown nowhere) and `INFRA_VAULT`
  through `gh`; and opens a pull request with a workflow that calls the shared one of
  `mirrorboards/mirrorboard-deploy`. A second run writes only what is missing. `--preview` shows
  the steps and does nothing.

### Fixed

- `xx mirrorboard deploy` and its `--preview` read a stack that was made but never deployed, whose
  export has no resources, instead of refusing it.

## [0.5.0] - 2026-11-27

### Added

- `xx wallet update_company_account <ACCOUNT> --identity <VAULT>/<IDENTITY>
  --member <ACCOUNT>=<WEIGHT>... --threshold <N> [--yes]`: new weights and a new threshold for a
  company's members, in its owner and active alike. A member who signs alone now signs, the
  company pays, and a change after which nobody signs alone asks first.
- `xx billing usage|room|dues|records|settlements`: what XBilling charges your account, or with
  `--as` a company's. `usage` sums the daily dues of whole UTC days by kind; `room <ROOM>` shows
  a room's life and its hourly storage as XBilling sampled it, beside the bytes its members read
  from XGit and XSQL now; `dues` prices one day line by line; `records [--room <ROOM>]` and
  `settlements` read as in `x billing`.
- `xx mirrorboard sign` approves together for a company in which no member signs alone. The first
  member puts the proposal's operations into a proposal of the chain, which lives a week, and
  approves it; each later member has the provider's own code check those operations against the
  proposal before it adds its approval, and the member who meets the threshold reports the
  outcome. The list shows the approvals so far and what each member pays. It needs an XSystem
  that collects joint approvals (acta-network's main-4b71dca).
- `xx mirrorboard withdraw <CODE> --identity <VAULT>/<IDENTITY> [--approved] [--yes]`: infra
  withdraws the open proposals of a deployment that no run will finish, as a trial's. Proposals
  that wait or collect their members' approvals go; approved ones only with `--approved`, since
  their owners may be carrying them out. Only the account that opened the deployment withdraws in
  it. One question, no by default; `--yes` without a terminal. A collecting proposal's proposal of
  the chain stays on the chain until it expires.
- A release carries xx for Linux x64 too, `xx-linux-x64.tar.gz`. It is built on Debian bookworm,
  so it runs where glibc is 2.36 or newer, and it needs libfido2 from the system (Debian, Ubuntu:
  `apt install libfido2-1`). `install.sh` and `xx update` take the archive of their system.

### Changed

- `xx mirrorboard deploy` brings what it runs: xx itself is the mirrorboard provider (Pulumi
  starts it as `pulumi-resource-mirrorboard` from a folder first in its PATH), and Pulumi is the
  version xx pins, fetched once into `~/.local/share/xx/pulumi` and checked by sha256 when the
  one on PATH is another version. cargo is checked before the run.
- A release carries Pulumi's Rust language host, `pulumi-language-rust`, beside xx, under the same
  signature; `install.sh` and `xx update` put it beside xx, and `xx mirrorboard deploy` puts it
  first in Pulumi's PATH. The `xx update` of 0.4.0 or older replaces xx alone: `install.sh` once
  puts the language host beside it.
- `xx mirrorboard deploy` reads again, before a run, the rooms whose members changed on the chain,
  so the state shows who is in them; a room with attention only where its owner's session runs.
  A member let in outside the program is named in a plain line and left in, and GitHub gets it in
  the summary without a warning.
- `xx release publish` signs every archive of a release, each with its own `.sig`, and refuses a
  draft that lacks one. Every archive also carries `pulumi.toml`, the Pulumi xx runs, which the
  deployment image installs ahead.
- `install.sh` installs only an archive whose signature the release carries and names it, and
  nothing that does not start on the computer.

### Fixed

- `xx mirrorboard sign` lets a member refuse a proposal that no account here may carry out, e.g.
  one for a room deleted since it was proposed. Such a proposal stays on the list with its reason:
  a yes leaves it alone, `n` and `--refuse` refuse it, and a list of only such proposals asks
  `Refuse all? [y/N]`.
- `xx mirrorboard deploy` names drift that the deployment's own proposals made as theirs. A member
  that a proposal signed after a waiting run let in, took out or changed is told as "let in by
  this deployment's proposal" (and so on), not as let in outside the program or changed by hand,
  and it no longer goes into the GitHub summary's "Also in the rooms".
- `xx mirrorboard deploy` ends a waiting run with the same line of each open proposal as it
  printed during the run: a proposal a company's members approve together reads "the members of
  <company> approve it together (<who> so far)" at the end too, not only "waits for <company>".

## [0.4.0] - 2026-11-27

### Added

- `xx git upload <ROOM> <FILE>... [--to <DIR>] [-m <TEXT>] [--branch <NAME>]`: files and folders of
  this computer go into a room's repository on XGit as one commit, without a clone; a file over
  1 MiB travels as a content object in 4 MiB chunks. The room's keys stay in the vault session.
- `xx git files <ROOM> [PATH]`: the files of a branch with the sizes XGit stores.
- `xx git clone <ROOM> [DIR]`, `xx git install`, `xx git retain`, `xx git objects fetch`: plain git
  with rooms for a person; the clone remembers who acts (`git config xx.identity`, `xx.as`), and
  `git-remote-xgit` is a link to xx, which `install.sh` makes too.

- `xx sql query|exec|export|snapshot <ROOM>`: a room's SQLite database for a person. The room is
  `1.24.N` or `<OWNER>/<NAME>`; without `--identity` the only identity here with access acts,
  for its own account or, with `--as` or when it is the only one, for a company it signs for
  alone. `query` prints a framed table, `-o json` an object per row or `-o csv`; a locked vault
  opens for the command only, at a terminal.

- A vault session started by an older xx reopens with this one at the terminal, for the time it
  had left; `xx vault list` shows it as `Unlocked (old)`, and `xx update` locks every session,
  as they run the old program.

- `xx wallet create_account <VAULT>/<NAME> --registrar <VAULT>/<IDENTITY> [--fund <AMOUNT>]`: the
  identity (made if the vault lacks it) becomes an account with its owner, active and memo keys,
  bound to it in the vault, and `--fund` sends it MBT from the registrar.
- `xx wallet link_identity <VAULT>/<IDENTITY> <ACCOUNT>`: binds an existing account after checking
  its keys on chain, so `list_my_accounts` and the identity show it.
- `xx wallet create_account … --registrar <URL>`: an XSystem's registrar registers the identity
  and pays; the vault signs only the request, and `--fund` is a top-up from that registrar.
- `xx wallet create_company_account <NAME> --founder <VAULT>/<IDENTITY> --member <ACCOUNT>=<WEIGHT>…
  --threshold <N> --registrar <URL>`: an XSystem's registrar founds a company account whose owner
  and active are the members with their weights and the threshold.
- `xx mirrorboard deploy [PATH] --identity <VAULT>/<IDENTITY>`: infra deploys a mirrorboard
  instance; a room nobody here signs for waits for its owner, and the command ends with exit
  status 3 while proposals wait. The stack's passphrase lives in an XKeys slot of infra's identity.
- `xx mirrorboard sign <HOST>/<CODE>`: the owner sees what a deployment waits for from their
  accounts, answers once for the whole list, and signs it in their own vault session.
- A room's members are resources of their own (`mirrorboard:index:RoomMember`). `sign` shows who
  gets into which room, with which flags, and who pays.
- A proposal has several owners, any one of whom signs it for all: the room's owner, and for a
  member also the members the chain lets act for the room's members. `deploy` shows
  "waits for a or b" and who signed.
- `xx mirrorboard sign --parallel <N>` carries out that many proposals of a list at once (50 by
  default). A journal keeps what `sign` did, so a proposal signed while XSystem was away is
  finished later. `--settle-unknown` settles the proposals approved earlier whose outcome is
  unknown here by what the chain shows.
- `xx mirrorboard deploy` makes the first stack of a new instance itself. Before each run it reads
  again the members changed by hand on the chain, so the run brings them back to the program, and
  it names the members of the stack's rooms that the program does not declare, leaving them in.
  After a run it lists only what the run changed, and counts the rest in one line,
  `N resource(s) unchanged`; in `-o json`, `result` says what was done.
- `xx mirrorboard passphrase add [PATH] --identity <VAULT>/<IDENTITY>`: one more identity, e.g.
  a CI vault's, gets a slot of an instance's passphrase; only `.stacks/passphrase` is committed.
- For GitHub Actions: `xx mirrorboard deploy --github` (job summary, notice or error, step
  outputs), `--only-when-signed` (resume from cron only when something was signed),
  `--preview -o markdown` (the pull request comment), and `xx vault import … --unlock-for` (infra's
  vault from the repository's secrets, opened with its recovery code).
- `xx wallet create_content_card <PAYER> <AUTHOR> <ROOM> <FILE> --type <TYPE>`: a card of an
  encrypted room whose content, at most 64 MiB, the vault session seals with a new key; the
  sealed bytes go to the room's XGit (`--xgit`, or the network's) before the card goes to the
  chain.
- `xx wallet update_content_card <PAYER> <CALLER> <CARD> [--file <FILE>] [--type <TYPE>]
  [--description <TEXT>] [--expected-revision <N>]`: new content gets a new key in the room's
  current epoch, and the command names the grants the chain dropped, to issue again.
- `xx wallet read_content_card <CARD> <FILE|->`: the content of a card into a file (0600) or to
  standard output, for a member of its room holding the card's epoch, or for the holder of a grant
  of the card; `--identity` and `--as` choose who reads when several identities here could.

### Changed

- `xx mirrorboard sign` shows a fee exactly where the chain does not charge it by size.
- `xx wallet register_account` gives the new account the memo key of the identity named as
  `ACTIVE`, so memos work without `update_account_keys`; an `MBT…` key is still also the memo key.
- `xx wallet create_fee_grant` takes a budget per asset, `asset:total[:per_period]`, and
  `update_fee_grant --new-budgets` replaces them, as the chain restarted on 2026-11-26 keeps them;
  `--period-budget` is gone. Without `--operations` a grant leaves out `tab_create` and
  `tab_deposit`, which spend the sponsor's money.
- The built-in network `mirrorboards` is the Mirrornet mainnet restarted on 2026-11-26 (chain id
  `93c660a5…`), and it no longer carries XBilling's key.
- The room writes of `xx wallet` act for a company account the identity signs for alone: its
  room keys open through the company's memo key, which the XSystem keeps wrapped for the member.

### Fixed

- `xx wallet grant_content_card` for a card of a company's room: the session tried the epoch
  envelope with the person's memo key and refused it.

## [0.3.0] - 2026-11-25

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
- `xx wallet` governance: `create_witness`, `update_witness`, `create_committee_member`,
  `withdraw_vesting`, `create_worker`, `update_worker_votes`, `vote_for_witness`,
  `vote_for_committee_member`, `set_voting_proxy`, `set_desired_witness_and_committee_member_count`,
  `propose_parameter_change`, `propose_fee_change`, `approve_proposal`; transactions from JSON:
  `sign_transaction`, `sign_transaction2`, `add_transaction_signature` (send only with
  `--broadcast`, as in cli_wallet), `broadcast_transaction`; and `update_account_keys`, outside
  cli_wallet, to set an account's owner, active or memo key from a vault's identity.
- `xx wallet` assets, market and HTLCs: `create_asset`, `update_asset`, `update_asset_issuer`,
  `update_bitasset`, `update_asset_feed_producers`, `publish_asset_feed`, `issue_asset`,
  `reserve_asset`, `fund_asset_fee_pool`, `claim_asset_fee_pool`, `global_settle_asset`,
  `settle_asset`, `bid_collateral`, `whitelist_account`, `sell_asset`, `borrow_asset`,
  `borrow_asset_ext`, `cancel_order`, `htlc_create`, `htlc_redeem`, `htlc_extend`.
- `xx wallet` data rooms and fee grants, with room keys made in the vault: `create_data_room`,
  `update_data_room`, `delete_data_room`, `add_data_room_member`, `update_data_room_member`,
  `remove_data_room_member`, `rotate_data_room_key`, `revoke_content_card_grant`,
  `remove_content_card`, `grant_content_card` (the vault reseals the card's key), `create_fee_grant`, `update_fee_grant`, `revoke_fee_grant`.
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

[Unreleased]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/mirrorboards-cli/cli-xx/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mirrorboards-cli/cli-xx/releases/tag/v0.1.0
