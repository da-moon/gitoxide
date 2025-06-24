use super::args::Args;
use crate::error::Result;
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let g = globals.clone();
    crate::util::spawn_blocking(move || crate::sdk::hours::analyze(args, &g)).await?;
    Ok(())
}
