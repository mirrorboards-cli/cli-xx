//! `xx`: the successor of the `x` command line (`mirrorboards-cli/cli-spec`, `xx.typ`). The owner
//! defines its interface command by command; each command is a thin layer over the core it shares
//! with `x` (`cli-runtime`, the libraries of `xkeys-cli` and of the plugins), so both programs see
//! the same networks, contexts, vaults and sessions, and `x` keeps working beside it.
//!
//! - `command_line`: the parser, the dispatch, the error line and the exit codes.
//! - `commands`: one file per command, one directory per group.

mod command_line;
mod commands;

pub use command_line::main;
