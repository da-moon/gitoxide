use clap::Parser;

mod cmd;
mod error;
mod sdk;
mod util;

use crate::error::Result;
use clap::ValueEnum;
use log::LevelFilter;

/// Shared options available to all subcommands.
#[derive(Clone)]
pub struct Globals {
    pub since: Option<String>,
    pub until: Option<String>,
    pub json: bool,
    pub log_level: log::LevelFilter,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LogVerbosity {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogVerbosity> for LevelFilter {
    fn from(v: LogVerbosity) -> Self {
        match v {
            LogVerbosity::Error => LevelFilter::Error,
            LogVerbosity::Warn => LevelFilter::Warn,
            LogVerbosity::Info => LevelFilter::Info,
            LogVerbosity::Debug => LevelFilter::Debug,
            LogVerbosity::Trace => LevelFilter::Trace,
        }
    }
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
    #[arg(long, value_enum, default_value_t = LogVerbosity::Warn, help = "Logging level")]
    log_level: LogVerbosity,
    #[command(subcommand)]
    command: cmd::Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli {
        since,
        until,
        json,
        log_level,
        command,
    } = Cli::parse();
    let level: LevelFilter = log_level.into();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level.to_string())).init();
    let globals = Globals {
        since,
        until,
        json,
        log_level: level,
    };
    match command {
        cmd::Command::Hours(args) => cmd::hours::run(args, &globals).await,
        cmd::Command::CommitFrequency(args) => cmd::commit_frequency::run(args, &globals).await,
        cmd::Command::TimeOfDay(args) => cmd::time_of_day::run(args, &globals).await,
        cmd::Command::Churn(args) => cmd::churn::run(args, &globals).await,
        cmd::Command::CommitSize(args) => cmd::commit_size::run(args, &globals).await,
        cmd::Command::Frecency(args) => cmd::frecency::run(args, &globals).await,
        cmd::Command::Ownership(args) => cmd::ownership::run(args, &globals).await,
    }
}
