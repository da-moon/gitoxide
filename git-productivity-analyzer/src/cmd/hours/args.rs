use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to the repository
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
    /// Revision to analyze
    #[arg(long, default_value = "HEAD")]
    pub rev: String,
}
