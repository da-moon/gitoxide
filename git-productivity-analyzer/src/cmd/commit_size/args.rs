use crate::cmd::common::CommonArgs;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(
        long = "percentiles",
        value_delimiter = ',',
        value_parser = clap::value_parser!(f64),
        num_args = 1..
    )]
    pub percentiles: Option<Vec<f64>>,
}

crate::impl_from_args!(Args, crate::sdk::commit_size::Options { percentiles });
