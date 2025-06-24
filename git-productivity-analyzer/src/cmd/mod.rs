pub mod hours;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Estimate time spent on repository work
    Hours(hours::Args),
}
