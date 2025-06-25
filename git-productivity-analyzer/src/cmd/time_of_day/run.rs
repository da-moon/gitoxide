use super::args::Args;
use crate::error::Result;
use crate::sdk::{run_with_analyzer, time_of_day::Options};
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts: Options = args.into();
    run_with_analyzer(opts, globals, |a, hist| a.print_histogram(hist)).await
}
