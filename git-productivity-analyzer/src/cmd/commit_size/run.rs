use super::args::Args;
use crate::error::Result;
use crate::sdk::{commit_size::Options, run_with_analyzer};
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts: Options = args.into();
    opts.validate()?;
    run_with_analyzer(opts, globals, |a, summary| a.print_summary(summary)).await
}
