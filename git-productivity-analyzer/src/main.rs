use clap::Parser;

mod cmd;
mod error;

use crate::error::Result;

/// Shared options available to all subcommands.
pub struct Globals {
    pub since: Option<String>,
    pub until: Option<String>,
    pub json: bool,
}

#[derive(Debug, Parser)]
#[command(name = "git-productivity-analyzer")]
struct Cli {
    /// Start date for analysis (inclusive)
    #[arg(long)]
    since: Option<String>,
    /// End date for analysis (inclusive)
    #[arg(long)]
    until: Option<String>,
    /// Produce JSON output
    #[arg(long)]
    json: bool,
    #[command(subcommand)]
    command: cmd::Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli {
        since,
        until,
        json,
        command,
    } = Cli::parse();
    let globals = Globals { since, until, json };
    match command {
        cmd::Command::Hours(args) => cmd::hours::run(args, &globals).await,
    }
}
