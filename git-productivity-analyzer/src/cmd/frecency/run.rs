//! Execution entry point for the `frecency` subcommand.

use super::args::Args;
use crate::error::Result;
use crate::sdk::{frecency::Options, run_with_analyzer};
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    // convert CLI arguments into SDK options and delegate to the shared runner
    let opts: Options = args.into();
    run_with_analyzer(opts, globals, |a, res| a.print_scores(res)).await
}
