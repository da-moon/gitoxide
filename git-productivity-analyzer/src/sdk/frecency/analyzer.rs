//! Provide analysis of how recently and frequently files changed.
//!
//! Each commit contributes a score to the touched files. The score is the
//! product of a file-size penalty and an age-based weight. Small files changed
//! in recent commits therefore score highest.

use crate::{error::Result, Globals};
use clap::ValueEnum;
use miette::IntoDiagnostic;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
/// Sort order used when printing frecency scores.
pub enum Order {
    /// Sort from lowest score to highest.
    Ascending,
    /// Sort from highest score to lowest.
    Descending,
}

/// Seconds in one day used for age calculations.
pub(super) const SECONDS_PER_DAY: i64 = 86_400;
/// Default exponent applied to the age weight for quadratic decay.
pub const DEFAULT_AGE_EXPONENT: f64 = 2.0;
/// Default reference size in bytes for the size penalty computation.
pub const DEFAULT_SIZE_PENALTY_REF: f64 = 1024.0;

#[derive(Clone)]
/// Parameters controlling repository traversal and output formatting.
pub struct Options {
    /// Repository location and revision range.
    pub repo: crate::sdk::RepoOptions,
    /// Optional set of paths to include in the analysis.
    pub paths: Option<HashSet<PathBuf>>,
    /// Stop after visiting at most this many commits.
    pub max_commits: Option<usize>,
    /// Sort results according to this order.
    pub order: Order,
    /// If set, only print file paths without their scores.
    pub path_only: bool,
    /// Override the current time for deterministic scoring.
    pub now: Option<i64>,
    /// Age weighting exponent controlling decay speed.
    pub age_exp: f64,
    /// Reference size in bytes for the file size penalty.
    pub size_ref: f64,
}

#[derive(Clone)]
/// Analyze git history to compute file frecency scores.
pub struct Analyzer {
    /// Options provided by the caller.
    pub(super) opts: Options,
    /// Global flags controlling repository access and output.
    pub(super) globals: Globals,
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl Analyzer {
    /// Analyze the repository and return each file with its frecency score.
    pub fn analyze(self) -> Result<Vec<(PathBuf, f64)>> {
        if self.opts.size_ref <= 0.0 {
            return Err(miette::miette!("--size-ref must be positive"));
        }
        // Discover the repository and handle repositories without commits
        // gracefully by returning an empty result instead of an error.
        let repo = gix::discover(&self.opts.repo.working_dir).into_diagnostic()?;
        match repo.head_id() {
            Ok(_) => {}
            Err(gix::reference::head_id::Error::PeelToId(gix::head::peel::into_id::Error::Unborn { .. })) => {
                return Ok(Vec::new())
            }
            Err(e) => return Err(miette::Report::msg(e.to_string())),
        }

        let start = crate::sdk::resolve_start_commit(&repo, &self.opts.repo.rev_spec, self.globals.until.as_deref())?;
        let since = crate::sdk::resolve_since_commit(&repo, self.globals.since.as_deref())?;

        // Use either the injected timestamp or the current time to weight commits.
        let now = self.opts.now.unwrap_or_else(Self::current_timestamp);

        // Gather scores for all files touched in the selected commits.
        let scores = self.collect_scores(&repo, start, since.as_ref(), now)?;

        // Sort results according to the CLI flags and return them.
        Ok(self.sort_scores(scores))
    }

    /// Sort the score map according to user preference.
    fn sort_scores(&self, scores: HashMap<PathBuf, f64>) -> Vec<(PathBuf, f64)> {
        // Convert map into a sortable vector.
        let mut out: Vec<_> = scores.into_iter().collect();

        // Determine desired ordering once before sorting.
        let desc = matches!(self.opts.order, Order::Descending);
        out.sort_by(|a, b| {
            use std::cmp::Ordering;
            let primary = if desc { b.1.total_cmp(&a.1) } else { a.1.total_cmp(&b.1) };
            if primary == Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                primary
            }
        });
        out
    }

    /// Return the current time as seconds since the Unix epoch.
    fn current_timestamp() -> i64 {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_secs() as i64,
            Err(_) => 0,
        }
    }

    // Printing logic is implemented in `printer.rs` to keep this file focused on
    // the scoring algorithm.
}

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Vec<(PathBuf, f64)>;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
