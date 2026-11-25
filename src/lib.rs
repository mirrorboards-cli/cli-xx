//! `xx`: the successor of the `x` command line (`mirrorboards-cli/cli-spec`, `xx.typ`). The owner
//! defines its interface command by command; each command is a thin layer over the core it shares
//! with `x` (`cli-runtime`, the libraries of `xkeys-cli` and of the plugins), so both programs see
//! the same networks, contexts, vaults and sessions, and `x` keeps working beside it.
//!
//! - `command_line`: the parser, the dispatch, the error line and the exit codes.
//! - `commands`: xx's own commands, one file each.
//! - `completion`: Tab completion, with the vaults, identities and networks of this computer.
//!
//! A tool's group of commands lives in the tool's crate, in its `xx` module (`xx vault` in
//! `xkeys-cli`), beside `x`'s own command line there; this crate mounts the groups. Retiring `x`
//! later removes its command lines and keeps the `xx` modules.

mod command_line;
mod commands;
mod completion;

pub use command_line::main;
pub mod release_signature;
