use super::args::Args;
use crate::error::Result;
use crate::sdk::{run_with_analyzer, streaks::Options};
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts: Options = args.into();
    run_with_analyzer(opts, globals, |a, streaks| a.print_streaks(streaks)).await
}
