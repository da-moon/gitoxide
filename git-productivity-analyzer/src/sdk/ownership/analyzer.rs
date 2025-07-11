use std::collections::BTreeMap;

use crate::{error::Result, Globals};
use gix::bstr::ByteSlice;
use glob::Pattern;
use miette::IntoDiagnostic;

#[derive(Clone)]
pub struct Options {
    pub repo: crate::sdk::RepoOptions,
    pub path: Option<String>,
    pub author: Option<String>,
    pub depth: usize,
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

pub type Totals = BTreeMap<String, BTreeMap<String, u32>>;

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl Analyzer {
    pub fn analyze(self) -> Result<Totals> {
        let (repo, start, since) = crate::sdk::open_with_range(&self.opts.repo, &self.globals)?;
        let pattern = match &self.opts.path {
            Some(p) => Some(Pattern::new(p).map_err(|e| miette::miette!(e.to_string()))?),
            None => None,
        };
        let mut totals = BTreeMap::<String, BTreeMap<String, u32>>::new();
        crate::sdk::walk_commits(&repo, start, since.as_ref(), false, |id, commit| {
            let author = commit.author().into_diagnostic()?;
            if !crate::sdk::author_matches_optimized(&author, &self.opts.author) {
                return Ok(());
            }
            let author_string = format!("{} <{}>", author.name, author.email);
            self.process_commit(&repo, id, &commit, pattern.as_ref(), &author_string, &mut totals)
        })?;
        Ok(totals)
    }

    fn process_commit(
        &self,
        repo: &gix::Repository,
        id: gix::ObjectId,
        commit: &gix::Commit<'_>,
        pattern: Option<&Pattern>,
        author: &str,
        totals: &mut Totals,
    ) -> Result<()> {
        // Compare merge commits against their first parent only to
        // avoid double-counting changes from multiple branches.
        let parent = commit.parent_ids().next().map(|p| p.detach());
        let (from, to) = crate::sdk::diff::commit_trees(repo, id, parent);
        let mut changes = crate::sdk::diff::create_changes(&from)?;
        crate::sdk::diff::configure_changes(&mut changes);
        changes
            .for_each_to_obtain_tree(&to, |change| {
                use gix::object::tree::diff::Change::*;
                let (location, mode) = match change {
                    Addition {
                        location, entry_mode, ..
                    } => (location, entry_mode),
                    Deletion {
                        location, entry_mode, ..
                    } => (location, entry_mode),
                    Modification {
                        location, entry_mode, ..
                    } => (location, entry_mode),
                    Rewrite {
                        location, entry_mode, ..
                    } => (location, entry_mode),
                };
                if mode.is_tree() {
                    return Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue);
                }
                let path = location.to_str_lossy();
                if let Some(pat) = pattern {
                    if !pat.matches(&path) {
                        return Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue);
                    }
                }
                let depth = self.opts.depth;
                let dir = if depth == 0 || !path.contains('/') {
                    ".".to_string()
                } else {
                    path.split('/').take(depth).collect::<Vec<_>>().join("/")
                };
                let by_author = totals.entry(dir).or_default();
                *by_author.entry(author.to_string()).or_default() += 1;
                Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
            })
            .into_diagnostic()?;
        Ok(())
    }

    pub fn print_totals(&self, totals: &Totals) {
        crate::sdk::print_json_or(self.globals.json, &SerializableTotals::from(totals), || {
            for (dir, counts) in totals {
                let total: u32 = counts.values().sum();
                print!("{dir}: ");
                if total == 0 {
                    println!();
                    continue;
                }
                let mut pairs: Vec<_> = counts.iter().collect();
                pairs.sort_by(|(a, ac), (b, bc)| bc.cmp(ac).then_with(|| a.cmp(b)));
                let mut parts = Vec::new();
                for (author, count) in pairs {
                    let pct = if total == 0 {
                        0.0
                    } else {
                        (*count as f64) * 100.0 / total as f64
                    };
                    parts.push(format!("{author} {:.0}%", pct));
                }
                println!("{}", parts.join(" "));
            }
        });
    }
}

#[derive(serde::Serialize)]
struct SerializableTotals(BTreeMap<String, BTreeMap<String, f64>>);

impl From<&Totals> for SerializableTotals {
    fn from(t: &Totals) -> Self {
        let mut out = BTreeMap::new();
        for (dir, counts) in t {
            let total: u32 = counts.values().sum();
            let mut percents = BTreeMap::new();
            for (author, count) in counts {
                let pct = if total == 0 {
                    0.0
                } else {
                    (*count as f64) * 100.0 / total as f64
                };
                percents.insert(author.clone(), pct);
            }
            out.insert(dir.clone(), percents);
        }
        SerializableTotals(out)
    }
}

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Totals;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
