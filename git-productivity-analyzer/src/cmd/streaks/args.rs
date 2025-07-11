use crate::cmd::common::CommonArgs;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(
        long,
        help = "Only count commits whose author contains this substring (case-insensitive)."
    )]
    pub author: Option<String>,
}

crate::impl_from_args!(Args, crate::sdk::streaks::Options { }, { author => author | lowercase });
