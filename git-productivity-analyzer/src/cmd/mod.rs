pub mod commit_frequency;
pub mod hours;
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
}
