use super::args::Args;
use crate::error::Result;
use crate::sdk::commit_frequency::Options;
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts = Options {
        working_dir: args.working_dir,
        rev_spec: args.rev_spec,
        author: args.author,
    };
    let analyzer = opts.into_analyzer(globals.clone());
    let worker = analyzer.clone();
    let totals = crate::util::spawn_blocking(move || worker.analyze()).await?;
    analyzer.print_totals(&totals);
    Ok(())
}
