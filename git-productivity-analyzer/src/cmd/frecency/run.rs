use super::args::Args;
use crate::error::Result;
use crate::sdk::{frecency::Options, run_with_analyzer};
use crate::Globals;

/// Entry point for the `frecency` CLI command.
///
/// The heavy lifting is done by the [`Analyzer`] in the SDK layer. This
/// function merely converts parsed arguments and delegates to the common
/// `run_with_analyzer` helper.
pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    // Convert CLI arguments to SDK options and execute the analysis.
    let opts: Options = args.into();
    run_with_analyzer(opts, globals, |a, res| a.print_scores(res)).await
}
