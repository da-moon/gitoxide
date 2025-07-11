use chrono::{naive::IsoWeek, Datelike, NaiveDate};
use miette::IntoDiagnostic;
use std::collections::{BTreeMap, BTreeSet};

use crate::error::Result;

pub(crate) fn process_commit(
    commit: gix::Commit<'_>,
    author_filter: &Option<String>,
    days: &mut BTreeMap<NaiveDate, u32>,
    weeks: &mut BTreeMap<IsoWeek, u32>,
    by_author: &mut BTreeMap<String, BTreeSet<NaiveDate>>,
) -> Result<()> {
    let author = commit.author().into_diagnostic()?;
    let author_string = format!("{} <{}>", author.name, author.email);
    if !crate::sdk::author_matches_optimized(&author, author_filter) {
        return Ok(());
    }
    let ts = author.seconds();
    let date = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
        .ok_or_else(|| miette::miette!("invalid timestamp {ts}"))?
        .date_naive();
    *days.entry(date).or_insert(0) += 1;
    let week = date.iso_week();
    *weeks.entry(week).or_insert(0) += 1;
    by_author.entry(author_string).or_default().insert(date);
    Ok(())
}
