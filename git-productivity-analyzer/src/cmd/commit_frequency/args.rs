use crate::cmd::common::CommonArgs;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(long, help = "Only count commits whose author matches this pattern.")]
    pub author: Option<String>,
}

impl From<Args> for crate::sdk::commit_frequency::Options {
    fn from(a: Args) -> Self {
        Self {
            repo: a.common.into(),
            author: a.author.map(|s| s.to_lowercase()),
        }
    }
}
