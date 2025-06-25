use std::collections::{BTreeMap, BTreeSet};

use crate::{error::Result, Globals};
use chrono::{naive::IsoWeek, NaiveDate};
use gix::prelude::*;
use miette::IntoDiagnostic;
use std::path::PathBuf;

use super::processor::process_commit;

pub struct Totals {
    pub commits_per_day: BTreeMap<NaiveDate, u32>,
    pub commits_per_week: BTreeMap<IsoWeek, u32>,
    pub active_days_per_author: BTreeMap<String, BTreeSet<NaiveDate>>,
}

#[derive(Clone)]
pub struct Options {
    pub working_dir: PathBuf,
    pub rev_spec: String,
    pub author: Option<String>,
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

impl Analyzer {
    pub fn analyze(self) -> Result<Totals> {
        let repo = gix::discover(&self.opts.working_dir).into_diagnostic()?;
        let start = crate::sdk::resolve_start_commit(&repo, &self.opts.rev_spec, self.globals.until.as_deref())?;
        let since = crate::sdk::resolve_since_commit(&repo, self.globals.since.as_deref())?;

        let (mut daily, mut weekly, mut days_by_author) = (
            BTreeMap::<NaiveDate, u32>::new(),
            BTreeMap::<IsoWeek, u32>::new(),
            BTreeMap::<String, BTreeSet<NaiveDate>>::new(),
        );
        self.walk_commits(
            &repo,
            start,
            since.as_ref(),
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

    fn walk_commits(
        &self,
        repo: &gix::Repository,
        start: gix::ObjectId,
        since: Option<&gix::ObjectId>,
        days: &mut BTreeMap<NaiveDate, u32>,
        weeks: &mut BTreeMap<IsoWeek, u32>,
        by_author: &mut BTreeMap<String, BTreeSet<NaiveDate>>,
    ) -> Result<()> {
        let iter = start.ancestors(&repo.objects);
        let mut buf = Vec::new();
        for item in iter {
            let info = item.into_diagnostic()?;
            let commit = repo.objects.find_commit_iter(&info.id, &mut buf).into_diagnostic()?;
            process_commit(commit, &self.opts.author, days, weeks, by_author)?;
            if let Some(id) = since {
                if &info.id == id {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn print_totals(&self, totals: &Totals) {
        crate::sdk::print_json_or(self.globals.json, &SerializableTotals::from(totals), || {
            for (day, count) in &totals.commits_per_day {
                println!("{}: {count}", day);
            }
            for (week, count) in &totals.commits_per_week {
                println!("week {}-{:02}: {count}", week.year(), week.week());
            }
            for (author, days) in &totals.active_days_per_author {
                println!("{author} active days: {}", days.len());
            }
        });
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

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Totals;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
