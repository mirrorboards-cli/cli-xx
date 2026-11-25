//! Tab completion of `xx` (owner, 2026-11-25). The shell asks `xx` itself at every Tab
//! (`COMPLETE=<shell> xx …`, clap_complete's dynamic completion), so besides commands and flags it
//! offers what is on this computer: vault names, identities as `<VAULT>/<IDENTITY>`, and networks.
//! The values are attached here, by the arguments' value names, so the crates whose commands xx
//! mounts need not know about completion.

use std::ffi::OsStr;
use std::process::Command as Process;

use anyhow::{Context, Result, bail};
use clap::{Arg, Args, Command, ValueEnum};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};

#[derive(Args)]
pub struct Arguments {
    /// The shell to complete in
    shell: Shell,
}

#[derive(Clone, Copy, ValueEnum)]
enum Shell {
    Fish,
    Zsh,
    Bash,
}

impl Shell {
    fn name(self) -> &'static str {
        match self {
            Self::Fish => "fish",
            Self::Zsh => "zsh",
            Self::Bash => "bash",
        }
    }
}

/// `xx completion <SHELL>`: prints the script that makes the shell ask xx at a Tab.
pub fn run(arguments: Arguments) -> Result<()> {
    let program = std::env::current_exe().context("cannot find this program")?;
    let status = Process::new(program)
        .env("COMPLETE", arguments.shell.name())
        .status()
        .context("cannot run xx for its completion script")?;
    if !status.success() {
        bail!("xx did not print its completion script");
    }

    Ok(())
}

/// `command` with the values of this computer attached to the arguments that take them.
pub fn with_values(command: Command) -> Command {
    attach(command, &[])
}

fn attach(command: Command, path: &[String]) -> Command {
    let in_vault_group = path.first().is_some_and(|group| group == "vault");
    let names_a_new_vault = matches!(path.get(1).map(String::as_str), Some("create" | "import"));
    let takes_existing_vault = in_vault_group && !names_a_new_vault;

    let command = command.mut_args(|arg| attach_to_arg(arg, takes_existing_vault));

    let subcommand_names: Vec<String> = command
        .get_subcommands()
        .map(|subcommand| subcommand.get_name().to_owned())
        .collect();

    let mut command = command;
    for name in subcommand_names {
        let mut sub_path = path.to_vec();
        sub_path.push(name.clone());

        command = command.mut_subcommand(&name, |subcommand| attach(subcommand, &sub_path));
    }

    command
}

fn attach_to_arg(arg: Arg, takes_existing_vault: bool) -> Arg {
    let value_name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|name| name.as_str().to_owned())
        .unwrap_or_default();

    if arg.get_long() == Some("network") {
        return arg.add(ArgValueCompleter::new(networks));
    }

    match value_name.as_str() {
        "VAULT>/<IDENTITY" => arg.add(ArgValueCompleter::new(identities)),
        "VAULT>/<NAME" => arg.add(ArgValueCompleter::new(vaults_with_slash)),
        "NAME" if arg.is_positional() && takes_existing_vault => {
            arg.add(ArgValueCompleter::new(vaults))
        }
        _ => arg,
    }
}

fn vault_names() -> Vec<String> {
    let names = xkeys_cli::vaults::Vaults::default_location().and_then(|vaults| vaults.names());

    names.unwrap_or_default()
}

fn starting_with(typed: &OsStr, values: Vec<String>) -> Vec<CompletionCandidate> {
    let typed = typed.to_string_lossy();

    values
        .into_iter()
        .filter(|value| value.starts_with(typed.as_ref()))
        .map(CompletionCandidate::new)
        .collect()
}

fn vaults(typed: &OsStr) -> Vec<CompletionCandidate> {
    starting_with(typed, vault_names())
}

fn vaults_with_slash(typed: &OsStr) -> Vec<CompletionCandidate> {
    let with_slash = vault_names()
        .into_iter()
        .map(|name| format!("{name}/"))
        .collect();

    starting_with(typed, with_slash)
}

/// `<VAULT>/<IDENTITY>` for every identity the vaults' catalogs list; a vault never opened here
/// has no catalog yet and offers none.
fn identities(typed: &OsStr) -> Vec<CompletionCandidate> {
    let Ok(vaults) = xkeys_cli::vaults::Vaults::default_location() else {
        return Vec::new();
    };

    let mut qualified = Vec::new();
    for vault in vault_names() {
        let Ok(home) = vaults.named(&vault) else {
            continue;
        };
        let Ok(Some(catalog)) = xkeys_cli::catalog::read(&home) else {
            continue;
        };

        for identity in catalog.identities {
            qualified.push(format!("{vault}/{}", identity.label));
        }
    }

    starting_with(typed, qualified)
}

/// The built-in network and those of the config file.
fn networks(typed: &OsStr) -> Vec<CompletionCandidate> {
    let mut names = vec![cli_runtime::config::BUILT_IN_NETWORK.to_owned()];

    let file = cli_runtime::config::path_from_environment()
        .ok()
        .and_then(|path| cli_runtime::config::ConfigFile::read(&path).ok().flatten());
    if let Some(file) = file {
        for name in file.networks.keys() {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
    }

    starting_with(typed, names)
}
