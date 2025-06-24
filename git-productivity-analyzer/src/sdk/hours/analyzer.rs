use crate::cmd::hours::Args;
use crate::error::Result;
use gitoxide_core::hours::{estimate, Context};
use gix::bstr::ByteSlice;
use miette::IntoDiagnostic;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize, Default)]
pub(crate) struct Summary {
    pub(crate) total_hours: f32,
    pub(crate) total_8h_days: f32,
    pub(crate) total_commits: u32,
    pub(crate) total_authors: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_files: Option<[u32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_lines: Option<[u32; 3]>,
}

pub fn analyze(args: Args, globals: &crate::Globals) -> Result<()> {
    let mut out_buf = Vec::new();
    let spec = globals.until.as_deref().unwrap_or(&args.rev_spec);
    estimate(
        &args.working_dir,
        spec.as_bytes().as_bstr(),
        &mut gix::progress::Discard,
        Context {
            show_pii: args.show_pii,
            ignore_bots: args.no_bots,
            file_stats: args.file_stats,
            line_stats: args.line_stats,
            omit_unify_identities: args.omit_unify_identities,
            threads: args.threads,
            out: &mut out_buf,
        },
    )
    .map_err(|e| miette::Report::msg(e.to_string()))?;

    if globals.json {
        let out_str = std::str::from_utf8(&out_buf).into_diagnostic()?;
        let summary = super::parser::parse_summary(out_str);
        serde_json::to_writer(std::io::stdout(), &summary).into_diagnostic()?;
        println!();
    } else {
        std::io::stdout().write_all(&out_buf).into_diagnostic()?;
    }
    Ok(())
}
