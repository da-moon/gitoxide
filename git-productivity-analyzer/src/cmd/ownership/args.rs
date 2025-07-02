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
}

crate::impl_from_args!(Args, crate::sdk::ownership::Options { path, author });
