use std::collections::{BTreeMap, BTreeSet};

use crate::{cmd::commit_frequency::Args, error::Result, Globals};
use chrono::{naive::IsoWeek, NaiveDate};
use gix::bstr::ByteSlice;
use gix::prelude::*;
use miette::IntoDiagnostic;

use super::processor::process_commit;

pub struct Totals {
    pub commits_per_day: BTreeMap<NaiveDate, u32>,
    pub commits_per_week: BTreeMap<IsoWeek, u32>,
    pub active_days_per_author: BTreeMap<String, BTreeSet<NaiveDate>>,
}

pub fn analyze(args: Args, globals: &Globals) -> Result<Totals> {
    let repo = gix::discover(&args.working_dir).into_diagnostic()?;
    let start = resolve_start_commit(&repo, &args.rev_spec, globals)?;
    let since = resolve_since_commit(&repo, &globals.since)?;

    let (mut daily, mut weekly, mut days_by_author) = (
        BTreeMap::<NaiveDate, u32>::new(),
        BTreeMap::<IsoWeek, u32>::new(),
        BTreeMap::<String, BTreeSet<NaiveDate>>::new(),
    );
    walk_commits(
        &repo,
        start,
        since.as_ref(),
        &args.author,
        &mut daily,
        &mut weekly,
        &mut days_by_author,
    )?;

    Ok(Totals {
        commits_per_day: daily,
        commits_per_week: weekly,
        active_days_per_author: days_by_author,
    })
}

fn resolve_start_commit(repo: &gix::Repository, rev_spec: &str, globals: &Globals) -> Result<gix::ObjectId> {
    let spec = globals.until.as_deref().unwrap_or(rev_spec);
    Ok(repo
        .rev_parse_single(spec.as_bytes().as_bstr())
        .into_diagnostic()?
        .detach())
}

fn resolve_since_commit(repo: &gix::Repository, since: &Option<String>) -> Result<Option<gix::ObjectId>> {
    match since {
        Some(spec) => Ok(Some(
            repo.rev_parse_single(spec.as_bytes().as_bstr())
                .into_diagnostic()?
                .detach(),
        )),
        None => Ok(None),
    }
}

fn walk_commits(
    repo: &gix::Repository,
    start: gix::ObjectId,
    since: Option<&gix::ObjectId>,
    author_filter: &Option<String>,
    days: &mut BTreeMap<NaiveDate, u32>,
    weeks: &mut BTreeMap<IsoWeek, u32>,
    by_author: &mut BTreeMap<String, BTreeSet<NaiveDate>>,
) -> Result<()> {
    let iter = start.ancestors(&repo.objects);
    let mut buf = Vec::new();
    for item in iter {
        let info = item.into_diagnostic()?;
        let commit = repo.objects.find_commit_iter(&info.id, &mut buf).into_diagnostic()?;
        process_commit(commit, author_filter, days, weeks, by_author)?;
        if let Some(id) = since {
            if &info.id == id {
                break;
            }
        }
    }
    Ok(())
}

pub fn print_totals(json: bool, totals: &Totals) {
    if json {
        let ser = SerializableTotals::from(totals);
        let _ = serde_json::to_writer(std::io::stdout(), &ser).map(|_| println!());
    } else {
        for (day, count) in &totals.commits_per_day {
            println!("{}: {count}", day);
        }
        for (week, count) in &totals.commits_per_week {
            println!("week {}-{:02}: {count}", week.year(), week.week());
        }
        for (author, days) in &totals.active_days_per_author {
            println!("{author} active days: {}", days.len());
        }
    }
}

#[derive(serde::Serialize)]
struct SerializableTotals {
    commits_per_day: BTreeMap<String, u32>,
    commits_per_week: BTreeMap<String, u32>,
    active_days_per_author: BTreeMap<String, u32>,
}

impl From<&Totals> for SerializableTotals {
    fn from(t: &Totals) -> Self {
        Self {
            commits_per_day: t.commits_per_day.iter().map(|(d, c)| (d.to_string(), *c)).collect(),
            commits_per_week: t
                .commits_per_week
                .iter()
                .map(|(w, c)| (format!("{}-{:02}", w.year(), w.week()), *c))
                .collect(),
            active_days_per_author: t
                .active_days_per_author
                .iter()
                .map(|(a, set)| (a.clone(), set.len() as u32))
                .collect(),
        }
    }
}
