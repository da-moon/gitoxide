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

crate::impl_from_args_author!(Args, crate::sdk::churn::Options { per_file });
