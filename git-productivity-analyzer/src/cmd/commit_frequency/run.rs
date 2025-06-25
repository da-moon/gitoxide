use super::args::Args;
use crate::error::Result;
use crate::sdk::{commit_frequency::Options, run_with_analyzer};
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts: Options = args.into();
    run_with_analyzer(opts, globals, |a, totals| a.print_totals(totals)).await
}
