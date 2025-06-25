use clap::Parser;
use std::path::PathBuf;

mod cmd;
mod error;
pub mod sdk;

use crate::error::Result;

#[derive(Debug, Parser)]
#[command(name = "branch-client")]
struct Cli {
    #[arg(long, default_value = ".", help = "Path to the repository")]
    repo: PathBuf,
    #[command(subcommand)]
    command: cmd::Command,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let repo = sdk::repo::open_repo(&args.repo)?;
    match args.command {
        cmd::Command::List(a) => cmd::list::run(&repo, a),
        cmd::Command::Create(a) => cmd::create::run(&repo, a),
        cmd::Command::Delete(a) => cmd::delete::run(&repo, a),
        cmd::Command::Compare(a) => cmd::compare::run(&repo, a),
        cmd::Command::Cleanup(a) => cmd::cleanup::run(&repo, a),
    }
}
