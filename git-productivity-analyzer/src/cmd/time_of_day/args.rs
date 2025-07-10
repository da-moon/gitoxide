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

crate::impl_from_args_author!(Args, crate::sdk::time_of_day::Options { bins });
