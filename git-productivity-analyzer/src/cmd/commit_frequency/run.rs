use super::args::Args;
use crate::error::Result;
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let g = globals.clone();
    let totals = crate::util::spawn_blocking(move || crate::sdk::commit_frequency::analyze(args, &g)).await?;
    crate::sdk::commit_frequency::print_totals(globals.json, &totals);
    Ok(())
}
