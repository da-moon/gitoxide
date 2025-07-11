use chrono::NaiveDate;
use miette::IntoDiagnostic;
use std::collections::{BTreeMap, BTreeSet};

use crate::error::Result;

pub(crate) fn process_commit(
    commit: gix::Commit<'_>,
    author_filter: &Option<String>,
    by_author: &mut BTreeMap<String, BTreeSet<NaiveDate>>,
) -> Result<()> {
    let author = commit.author().into_diagnostic()?;
    if !crate::sdk::author_matches_optimized(&author, author_filter) {
        return Ok(());
    }
    let ts = author.seconds();
    let date = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
        .ok_or_else(|| miette::miette!("invalid timestamp {ts}"))?
        .date_naive();
    let key = format!("{} <{}>", author.name, author.email);
    by_author.entry(key).or_default().insert(date);
    Ok(())
}
