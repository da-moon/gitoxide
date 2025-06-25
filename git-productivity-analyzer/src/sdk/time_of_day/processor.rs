use chrono::{FixedOffset, Timelike, Utc};
use gix::bstr::ByteSlice;
use miette::IntoDiagnostic;

use crate::error::Result;

pub(crate) fn process_commit(
    commit: gix::objs::CommitRefIter<'_>,
    author_filter: &Option<String>,
    bins: &mut [u32],
) -> Result<()> {
    let author = commit.author().into_diagnostic()?;
    if let Some(pattern) = author_filter {
        let pat = pattern.as_str();
        if !author.name.to_str_lossy().contains(pat) && !author.email.to_str_lossy().contains(pat) {
            return Ok(());
        }
    }
    let time = author.time().into_diagnostic()?;
    let secs = time.seconds;
    let offset_seconds = time.offset;
    let offset = FixedOffset::east_opt(offset_seconds).ok_or_else(|| miette::miette!("invalid offset"))?;
    let dt_utc =
        chrono::DateTime::<Utc>::from_timestamp(secs, 0).ok_or_else(|| miette::miette!("invalid timestamp {secs}"))?;
    let dt = dt_utc.with_timezone(&offset);
    let hour = dt.hour();
    let bin = hour * bins.len() as u32 / 24;
    bins[bin as usize] += 1;
    Ok(())
}
