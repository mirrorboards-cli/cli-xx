//! The command line of `xx`: its commands, the line an error ends with and the exit codes, as in
//! `x` (cli-spec MR-17): 0 done, 1 failed with one line `xx: <message>`, 2 a mistyped command.

use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::commands::{update, version};

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
    /// Identities: keys under a name, in a vault
    #[command(subcommand)]
    Identity(xkeys_cli::xx::identity::Command),
    /// Pulumi platforms: deploy and preview their stacks
    #[command(subcommand)]
    Stacks(mirrorboards_xstacks::xx::stacks::Command),
    /// Vaults: where your identities' keys are kept
    #[command(subcommand)]
    Vault(xkeys_cli::xx::vault::Command),
    /// Update xx to its newest release on GitHub
    Update(update::Arguments),
    /// Print the version of xx
    Version,
}

pub fn main() -> ExitCode {
    cli_runtime::plugin::restore_default_sigpipe();

    let arguments = Arguments::parse();

    match run(arguments.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xx: {error:#}");

            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<()> {
    match command {
        // Synchronous: xstacks makes its own runtime for each call to a vault's session.
        Command::Stacks(command) => mirrorboards_xstacks::xx::stacks::run(command),
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
        Command::Update(arguments) => update::run(arguments),
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
}
