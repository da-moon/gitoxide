use crate::error::Result;
use gitoxide_core::hours::{estimate, Context};
use gix::bstr::ByteSlice;
use serde::Serialize;
use std::io::Write;

use super::args::Args;

#[derive(Serialize)]
struct Summary {
    total_hours: f32,
    total_8h_days: f32,
    total_commits: u32,
    total_authors: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_files: Option<[u32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_lines: Option<[u32; 3]>,
}

pub async fn run(args: Args, globals: &crate::Globals) -> Result<()> {
    let _since = globals.since.clone();
    let until = globals.until.clone();
    let json = globals.json;
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut progress = gix::progress::Discard;
        let mut out_buf = Vec::new();

        let mut rev_spec = args.rev_spec.clone();
        if let Some(u) = &until {
            rev_spec = u.clone();
        }

        estimate(
            &args.working_dir,
            rev_spec.as_bytes().as_bstr(),
            &mut progress,
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
        .map_err(crate::error::Error::from)?;

        if json {
            let out_str =
                std::str::from_utf8(&out_buf).map_err(|e| crate::error::Error::from(anyhow::Error::new(e)))?;
            let summary = parse_summary(out_str);
            serde_json::to_writer(std::io::stdout(), &summary)
                .map_err(|e| crate::error::Error::from(anyhow::Error::new(e)))?;
            println!();
        } else {
            std::io::stdout()
                .write_all(&out_buf)
                .map_err(|e| crate::error::Error::from(anyhow::Error::new(e)))?;
        }
        Ok(())
    })
    .await
    .map_err(|e| crate::error::Error::from(anyhow::Error::from(e)))??;
    Ok(())
}

fn parse_summary(out: &str) -> Summary {
    let mut summary = Summary {
        total_hours: 0.0,
        total_8h_days: 0.0,
        total_commits: 0,
        total_authors: 0,
        total_files: None,
        total_lines: None,
    };
    for line in out.lines() {
        if let Some(v) = line.strip_prefix("total hours: ") {
            summary.total_hours = v.trim().parse().unwrap_or_default();
        } else if let Some(v) = line.strip_prefix("total 8h days: ") {
            summary.total_8h_days = v.trim().parse().unwrap_or_default();
        } else if let Some(v) = line.strip_prefix("total commits = ") {
            let num = v.split_whitespace().next().unwrap_or("0");
            summary.total_commits = num.parse().unwrap_or_default();
        } else if let Some(v) = line.strip_prefix("total authors: ") {
            summary.total_authors = v.trim().parse().unwrap_or_default();
        } else if let Some(v) = line.strip_prefix("total files added/removed/modified/remaining: ") {
            let parts: Vec<u32> = v
                .split('/')
                .filter_map(|p| p.split_whitespace().next())
                .filter_map(|p| p.parse().ok())
                .collect();
            if parts.len() == 4 {
                summary.total_files = Some([parts[0], parts[1], parts[2], parts[3]]);
            }
        } else if let Some(v) = line.strip_prefix("total lines added/removed/remaining: ") {
            let parts: Vec<u32> = v
                .split('/')
                .filter_map(|p| p.split_whitespace().next())
                .filter_map(|p| p.parse().ok())
                .collect();
            if parts.len() == 3 {
                summary.total_lines = Some([parts[0], parts[1], parts[2]]);
            }
        }
    }
    summary
}
