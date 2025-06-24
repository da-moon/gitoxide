use super::args::Args;
use crate::error::Result;
use crate::Globals;
use miette::IntoDiagnostic;
use tokio::task;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let globals = globals.clone();
    task::spawn_blocking(move || crate::sdk::commit_frequency::analyze(args, &globals))
        .await
        .into_diagnostic()??;
    Ok(())
}
