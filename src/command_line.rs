//! The command line of `xx`: its commands, the line an error ends with and the exit codes, as in
//! `x` (cli-spec MR-17): 0 done, 1 failed with one line `xx: <message>`, 2 a mistyped command;
//! `xx mirrorboard deploy` also ends with 3 while its deployment waits for signatures.

use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use mirrornet_cli::xx::mirrorboard::WaitingForSignatures;

use crate::commands::{release, update, version};
use crate::completion;

/// The exit status of a deployment that waits for signatures (PX-9); it has printed what waits.
const WAITING_FOR_SIGNATURES: u8 = 3;

#[derive(Parser)]
#[command(
    name = "xx",
    version,
    about = "The successor of x, one command at a time; x keeps working beside it"
)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// What XBilling charges your account: usage, a room, the dues of a day, records, settlements
    #[command(subcommand)]
    Billing(xbilling_cli::xx::billing::Command),
    /// Rooms' git repositories: upload files, list them, clone a room for plain git
    #[command(subcommand)]
    Git(xgit_cli::xx::git::Command),
    /// Identities: keys under a name, in a vault
    #[command(subcommand)]
    Identity(xkeys_cli::xx::identity::Command),
    /// Mirrorboard instances: infra deploys them, owners sign their parts
    #[command(subcommand)]
    Mirrorboard(mirrornet_cli::xx::mirrorboard::Command),
    /// Rooms' SQLite databases: query them, change them, export or snapshot them
    #[command(subcommand)]
    Sql(xsql_cli::xx::sql::Command),
    /// Pulumi platforms: deploy and preview their stacks
    #[command(subcommand)]
    Stacks(mirrorboards_xstacks::xx::stacks::Command),
    /// Vaults: where your identities' keys are kept
    #[command(subcommand)]
    Vault(xkeys_cli::xx::vault::Command),
    /// Mirrornet: the methods of cli_wallet, with keys from your vaults
    #[command(subcommand)]
    Wallet(mirrornet_cli::xx::wallet::Command),
    /// Releases of xx: build, sign and publish one
    #[command(subcommand)]
    Release(release::Command),
    /// Update xx to its newest release on GitHub
    Update(update::Arguments),
    /// Print the version of xx
    Version,
    /// Print the script that completes xx at a Tab, e.g. xx completion fish > ~/.config/fish/completions/xx.fish
    Completion(completion::Arguments),
}

pub fn main() -> ExitCode {
    cli_runtime::plugin::restore_default_sigpipe();

    // Git starts its helper for xgit:// remotes by name; `xx git install` makes that name a link
    // to xx (SG-7 of cli-spec xx.typ).
    let arguments: Vec<String> = std::env::args().collect();
    if started_as(&arguments, xgit_cli::xx::git::HELPER_NAME) {
        return xgit_cli::xx::git::run_as_git_remote_helper(&arguments);
    }

    // Sessions this program starts report it by this version (`xx vault list`, `xx update`).
    xkeys_cli::session::program::register_version(env!("CARGO_PKG_VERSION"));

    // A Tab in the shell runs xx with COMPLETE set: it answers with candidates and exits here.
    clap_complete::CompleteEnv::with_factory(|| completion::with_values(Arguments::command()))
        .complete();

    let arguments = Arguments::parse();

    match run(arguments.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.is::<WaitingForSignatures>() => ExitCode::from(WAITING_FOR_SIGNATURES),
        Err(error) => {
            eprintln!("xx: {error:#}");

            ExitCode::FAILURE
        }
    }
}

/// Whether the program was started under the file name `name`.
fn started_as(arguments: &[String], name: &str) -> bool {
    let program = arguments.first().map(std::path::Path::new);

    program
        .and_then(|program| program.file_name())
        .is_some_and(|file_name| file_name == name)
}

fn run(command: Command) -> Result<()> {
    match command {
        // Synchronous: each command of xx git makes its own runtime, and the filter and the
        // helper block on theirs while git talks to them.
        Command::Git(command) => xgit_cli::xx::git::run(command),
        // Synchronous: xstacks makes its own runtime for each call to a vault's session.
        Command::Stacks(command) => mirrorboards_xstacks::xx::stacks::run(command),
        Command::Billing(command) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot start the async runtime")?;

            runtime.block_on(xbilling_cli::xx::billing::run(command))
        }
        Command::Identity(command) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot start the async runtime")?;

            runtime.block_on(xkeys_cli::xx::identity::run(command))
        }
        Command::Vault(command) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot start the async runtime")?;

            runtime.block_on(xkeys_cli::xx::vault::run(command))
        }
        Command::Mirrorboard(command) => {
            // One thread: the provider's code that `sign` runs is not `Send` (PX-12).
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot start the async runtime")?;

            runtime.block_on(mirrornet_cli::xx::mirrorboard::run(command))
        }
        Command::Sql(command) => {
            // One thread: the database work runs on a blocking thread, and the chain's connection
            // stays on this runtime meanwhile.
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot start the async runtime")?;

            runtime.block_on(xsql_cli::xx::sql::run(command))
        }
        Command::Wallet(command) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot start the async runtime")?;

            runtime.block_on(mirrornet_cli::xx::wallet::run(command))
        }
        Command::Release(command) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot start the async runtime")?;

            runtime.block_on(release::run(command))
        }
        Command::Update(arguments) => update::run(arguments),
        Command::Completion(arguments) => completion::run(arguments),
        Command::Version => {
            version::run();

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn version_is_a_command_and_a_mistyped_one_is_refused() {
        let parsed = Arguments::try_parse_from(["xx", "version"]).unwrap();
        let mistyped = Arguments::try_parse_from(["xx", "verison"]).err().unwrap();

        assert!(matches!(parsed.command, Command::Version));
        assert_eq!(mistyped.kind(), ErrorKind::InvalidSubcommand);
        assert_eq!(mistyped.exit_code(), 2);
    }

    #[test]
    fn xx_started_as_git_remote_xgit_is_gits_helper() {
        let as_helper = [
            "/usr/local/bin/git-remote-xgit",
            "origin",
            "xgit://x/1.24.1",
        ]
        .map(str::to_owned);
        let as_itself = ["/usr/local/bin/xx", "git", "files"].map(str::to_owned);

        assert!(started_as(&as_helper, "git-remote-xgit"));
        assert!(!started_as(&as_itself, "git-remote-xgit"));
        assert!(!started_as(&[], "git-remote-xgit"));
    }
}
