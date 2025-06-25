use super::args::Args;
use crate::error::Result;
use crate::sdk::time_of_day::{Analyzer, Options};
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts = Options {
        working_dir: args.working_dir,
        rev_spec: args.rev_spec,
        bins: args.bins,
        author: args.author,
    };
    let run_opts = opts.clone();
    let g = globals.clone();
    let hist = crate::util::spawn_blocking(move || run_opts.into_analyzer(g).analyze()).await?;
    opts.into_analyzer(globals.clone()).print_histogram(&hist);
    Ok(())
}
