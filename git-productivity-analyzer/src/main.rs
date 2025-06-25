use clap::Parser;

mod cmd;
mod error;
mod sdk;
mod util;

use crate::error::Result;

/// Shared options available to all subcommands.
#[derive(Clone)]
pub struct Globals {
    pub since: Option<String>,
    pub until: Option<String>,
    pub json: bool,
}

#[derive(Debug, Parser)]
#[command(name = "git-productivity-analyzer")]
struct Cli {
    #[arg(long, help = "Start date for analysis (inclusive)")]
    since: Option<String>,
    #[arg(long, help = "End date for analysis (inclusive)")]
    until: Option<String>,
    #[arg(long, help = "Produce JSON output")]
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
        cmd::Command::CommitFrequency(args) => cmd::commit_frequency::run(args, &globals).await,
        cmd::Command::TimeOfDay(args) => cmd::time_of_day::run(args, &globals).await,
    }
}
