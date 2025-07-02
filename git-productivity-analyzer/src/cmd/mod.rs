pub mod churn;
pub mod commit_frequency;
pub mod commit_size;
pub mod common;
pub mod frecency;
pub mod hours;
pub mod ownership;
pub mod time_of_day;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Estimate time spent on repository work")]
    Hours(hours::Args),
    #[command(about = "Count commits per day and week")]
    CommitFrequency(commit_frequency::Args),
    #[command(about = "Histogram of commit times across the day")]
    TimeOfDay(time_of_day::Args),
    #[command(about = "Summarize lines added and removed")]
    Churn(churn::Args),
    #[command(about = "Analyze distribution of commit sizes")]
    CommitSize(commit_size::Args),
    #[command(about = "Score files by recent change frequency")]
    Frecency(frecency::Args),
    #[command(about = "Summarize code ownership by directory")]
    Ownership(ownership::Args),
}
