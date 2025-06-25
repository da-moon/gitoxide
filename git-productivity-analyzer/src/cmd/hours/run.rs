use super::args::Args;
use crate::error::Result;
use crate::sdk::{hours::Options, run_with_analyzer};
use crate::Globals;

pub async fn run(args: Args, globals: &Globals) -> Result<()> {
    let opts: Options = args.into();
    run_with_analyzer(opts, globals, |_, _| {}).await
}
