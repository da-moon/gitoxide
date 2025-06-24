pub mod commit_frequency;
pub mod hours;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Estimate time spent on repository work")]
    Hours(hours::Args),
    #[command(about = "Count commits per day and week")]
    CommitFrequency(commit_frequency::Args),
}
