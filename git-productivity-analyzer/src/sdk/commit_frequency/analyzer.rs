use std::collections::{BTreeMap, BTreeSet};

use crate::{cmd::commit_frequency::Args, error::Result, Globals};
use gix::bstr::ByteSlice;
use gix::prelude::*;
use miette::IntoDiagnostic;

use super::processor::process_commit;

#[derive(serde::Serialize)]
struct Totals {
    commits_per_day: BTreeMap<String, u32>,
    commits_per_week: BTreeMap<String, u32>,
    active_days_per_author: BTreeMap<String, u32>,
}

pub fn analyze(args: Args, globals: &Globals) -> Result<()> {
    let repo = gix::discover(&args.working_dir).into_diagnostic()?;
    let start = resolve_start_commit(&repo, &args.rev_spec, globals)?;
    let since = resolve_since_commit(&repo, &globals.since)?;

    let (mut daily, mut weekly, mut days_by_author) = (BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    walk_commits(
        &repo,
        start,
        since.as_ref(),
        &args.author,
        &mut daily,
        &mut weekly,
        &mut days_by_author,
    )?;

    output_results(globals.json, daily, weekly, days_by_author);
    Ok(())
}

fn resolve_start_commit<'repo>(
    repo: &'repo gix::Repository,
    rev_spec: &str,
    globals: &Globals,
) -> Result<gix::ObjectId> {
    let spec = globals.until.as_deref().unwrap_or(rev_spec);
    Ok(repo
        .rev_parse_single(spec.as_bytes().as_bstr())
        .into_diagnostic()?
        .detach())
}

fn resolve_since_commit<'repo>(repo: &'repo gix::Repository, since: &Option<String>) -> Result<Option<gix::ObjectId>> {
    match since {
        Some(spec) => Ok(Some(
            repo.rev_parse_single(spec.as_bytes().as_bstr())
                .into_diagnostic()?
                .detach(),
        )),
        None => Ok(None),
    }
}

fn walk_commits<'repo>(
    repo: &'repo gix::Repository,
    start: gix::ObjectId,
    since: Option<&gix::ObjectId>,
    author_filter: &Option<String>,
    days: &mut BTreeMap<String, u32>,
    weeks: &mut BTreeMap<String, u32>,
    by_author: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    let mut iter = start.ancestors(&repo.objects);
    while let Some(item) = iter.next() {
        let info = item.into_diagnostic()?;
        let commit = iter.commit_iter();
        process_commit(commit, author_filter, days, weeks, by_author)?;
        if let Some(id) = since {
            if &info.id == id {
                break;
            }
        }
    }
    Ok(())
}

fn output_results(
    json: bool,
    commits_per_day: BTreeMap<String, u32>,
    commits_per_week: BTreeMap<String, u32>,
    days_by_author: BTreeMap<String, BTreeSet<String>>,
) {
    if json {
        let totals = Totals {
            commits_per_day,
            commits_per_week,
            active_days_per_author: days_by_author.into_iter().map(|(k, v)| (k, v.len() as u32)).collect(),
        };
        let _ = serde_json::to_writer(std::io::stdout(), &totals).and_then(|_| {
            println!();
            Ok(())
        });
    } else {
        for (day, count) in &commits_per_day {
            println!("{day}: {count}");
        }
        for (week, count) in &commits_per_week {
            println!("week {week}: {count}");
        }
        for (author, days) in &days_by_author {
            println!("{author} active days: {}", days.len());
        }
    }
}
