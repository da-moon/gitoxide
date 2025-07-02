use crate::cmd::common::CommonArgs;
use clap::{Args as ClapArgs, ValueHint};

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(long, value_hint = ValueHint::FilePath, help = "Only include paths matching this glob")]
    pub path: Option<String>,

    #[arg(long, help = "Only count commits whose author matches this pattern.")]
    pub author: Option<String>,

    #[arg(
        long,
        default_value_t = 1,
        help = "Number of path segments to group by when summarizing ownership.\nUse 0 to group all files together."
    )]
    pub depth: usize,
}

crate::impl_from_args!(Args, crate::sdk::ownership::Options { path, author, depth });
