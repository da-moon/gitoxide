use crate::cmd::common::CommonArgs;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(long, help = "Show totals per file path instead of per author.")]
    pub per_file: bool,

    #[arg(long, help = "Only count commits whose author matches this pattern.")]
    pub author: Option<String>,
}

impl From<Args> for crate::sdk::churn::Options {
    fn from(a: Args) -> Self {
        Self {
            repo: a.common.into(),
            per_file: a.per_file,
            author: a.author.map(|s| s.to_lowercase()),
        }
    }
}
