use crate::cmd::common::CommonArgs;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(long, default_value_t = 24, help = "Number of bins for the 24h day")]
    pub bins: u8,

    #[arg(long, help = "Only count commits whose author matches this pattern.")]
    pub author: Option<String>,
}

impl From<Args> for crate::sdk::time_of_day::Options {
    fn from(a: Args) -> Self {
        Self {
            repo: a.common.into(),
            bins: a.bins,
            author: a.author.map(|s| s.to_lowercase()),
        }
    }
}
