use crate::error::Result;
use gitoxide_core::hours::{estimate, Context};
use gix::bstr::ByteSlice;

use super::args::Args;

pub async fn run(args: Args, _globals: &crate::Globals) -> Result<()> {
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut progress = gix::progress::Discard;
        let stdout = std::io::stdout();
        let out = std::io::BufWriter::new(stdout);
        estimate(
            &args.repo,
            args.rev.as_bytes().as_bstr(),
            &mut progress,
            Context {
                show_pii: true,
                ignore_bots: true,
                file_stats: false,
                line_stats: false,
                omit_unify_identities: false,
                threads: None,
                out,
            },
        )
        .map_err(crate::error::Error::from)
    })
    .await
    .map_err(|e| crate::error::Error::from(anyhow::Error::from(e)))??;
    Ok(())
}
