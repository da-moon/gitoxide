use chrono::Datelike;
use gix::bstr::ByteSlice;
use gix::prelude::*;
use miette::IntoDiagnostic;
use std::collections::{BTreeMap, BTreeSet};

use crate::error::Result;

pub(crate) fn process_commit(
    mut commit: gix::objs::CommitRefIter<'_>,
    author_filter: &Option<String>,
    days: &mut BTreeMap<String, u32>,
    weeks: &mut BTreeMap<String, u32>,
    by_author: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    let author = commit.author().into_diagnostic()?;
    let author_string = format!("{} <{}>", author.name, author.email);
    if let Some(pattern) = author_filter {
        let pat = pattern.as_str();
        if !author.name.to_str_lossy().contains(pat) && !author.email.to_str_lossy().contains(pat) {
            return Ok(());
        }
    }
    let ts = author.seconds();
    let date = chrono::NaiveDateTime::from_timestamp_opt(ts as i64, 0).unwrap().date();
    let day = date.to_string();
    *days.entry(day.clone()).or_insert(0) += 1;
    let iso_week = date.iso_week();
    let week = format!("{}-{:02}", iso_week.year(), iso_week.week());
    *weeks.entry(week).or_insert(0) += 1;
    by_author.entry(author_string).or_default().insert(day);
    Ok(())
}
