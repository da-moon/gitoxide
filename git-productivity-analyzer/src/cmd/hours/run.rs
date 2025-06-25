use super::args::Args;
use crate::error::Result;
use crate::sdk::hours::Options;
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts = Options {
        working_dir: args.working_dir,
        rev_spec: args.rev_spec,
        no_bots: args.no_bots,
        file_stats: args.file_stats,
        line_stats: args.line_stats,
        show_pii: args.show_pii,
        omit_unify_identities: args.omit_unify_identities,
        threads: args.threads,
    };
    let g = globals.clone();
    crate::util::spawn_blocking(move || opts.into_analyzer(g).analyze()).await?;
    Ok(())
}
