use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(
        long = "working-dir",
        default_value = ".",
        help = "The directory containing a '.git/' folder."
    )]
    pub working_dir: PathBuf,

    #[arg(
        long = "rev-spec",
        default_value = "HEAD",
        help = "The revision to start walking from."
    )]
    pub rev_spec: String,

    #[arg(long, default_value_t = 24, help = "Number of bins for the 24h day")]
    pub bins: u8,

    #[arg(long, help = "Only count commits whose author matches this pattern.")]
    pub author: Option<String>,
}
