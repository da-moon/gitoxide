//! Implementation of the `frecency` subcommand.
//!
//! This module wires together argument parsing and the async `run` entry point
//! so it mirrors the layout of the other commands.

mod args;
mod run;

pub use args::Args;
pub use run::run;
