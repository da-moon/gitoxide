use chrono::NaiveDate;
use std::collections::{BTreeMap, BTreeSet};

use super::processor::process_commit;
use crate::{error::Result, Globals};

#[derive(Clone, serde::Serialize)]
pub struct Streaks(BTreeMap<String, u32>);

impl From<BTreeMap<String, u32>> for Streaks {
    fn from(map: BTreeMap<String, u32>) -> Self {
        Self(map)
    }
}

impl Streaks {
    pub fn into_inner(self) -> BTreeMap<String, u32> {
        self.0
    }

    pub fn as_map(&self) -> &BTreeMap<String, u32> {
        &self.0
    }
}

#[derive(Clone)]
pub struct Options {
    pub repo: crate::sdk::RepoOptions,
    pub author: Option<String>,
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

impl Analyzer {
    pub fn analyze(self) -> Result<Streaks> {
        let (repo, start, since) = crate::sdk::open_with_range(&self.opts.repo, &self.globals)?;
        let mut days_by_author = BTreeMap::<String, BTreeSet<NaiveDate>>::new();
        self.walk_commits(&repo, start, since.as_ref(), &mut days_by_author)?;

        let map: BTreeMap<_, _> = days_by_author
            .into_iter()
            .map(|(author, days)| (author, longest_streak(&days)))
            .collect();
        Ok(map.into())
    }

    fn walk_commits(
        &self,
        repo: &gix::Repository,
        start: gix::ObjectId,
        since: Option<&gix::ObjectId>,
        by_author: &mut BTreeMap<String, BTreeSet<NaiveDate>>,
    ) -> Result<()> {
        crate::sdk::walk_commits(repo, start, since, false, |_, commit| {
            process_commit(commit, &self.opts.author, by_author)
        })
    }

    pub fn print_streaks(&self, streaks: &Streaks) {
        crate::sdk::print_json_or(self.globals.json, streaks, || {
            for (author, days) in streaks.as_map() {
                println!("{author}: {days}");
            }
        });
    }
}

fn longest_streak(days: &BTreeSet<NaiveDate>) -> u32 {
    let mut max = 0;
    let mut current = 0;
    let mut prev: Option<NaiveDate> = None;
    for day in days {
        if matches!(prev, Some(p) if p.succ_opt() == Some(*day)) {
            current += 1;
        } else {
            current = 1;
        }
        max = max.max(current);
        prev = Some(*day);
    }
    max
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Streaks;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
