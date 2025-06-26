//! Provide analysis of how recently and frequently files changed.
//!
//! Each commit contributes a score to the touched files. The score is the
//! product of a file-size penalty and an age-based weight. Small files changed
//! in recent commits therefore score highest.

use crate::{error::Result, Globals};
use clap::ValueEnum;
use gix::bstr::ByteSlice;
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
const SECONDS_PER_DAY: i64 = 86_400;
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

    /// Walk commits and accumulate scores for changed files.
    fn collect_scores(
        &self,
        repo: &gix::Repository,
        start: gix::ObjectId,
        since: Option<&gix::ObjectId>,
        now: i64,
    ) -> Result<HashMap<PathBuf, f64>> {
        // Track frecency score per file path.
        let mut scores = HashMap::new();
        let mut size_cache = HashMap::new();

        // Optional commit limit to reduce execution time on large repositories.
        let limit = self.opts.max_commits.unwrap_or(usize::MAX);
        let mut count = 0usize;
        crate::sdk::walk_commits(repo, start, since, |id, commit| {
            // Stop iteration once the desired number of commits has been seen.
            if count >= limit {
                return Ok(());
            }

            // Accumulate frecency contributions from this commit.
            self.process_commit(repo, id, &commit, now, &mut scores, &mut size_cache)?;
            count += 1;
            Ok(())
        })?;
        Ok(scores)
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

    /// Inspect a single commit and add its contributions to `scores`.
    fn process_commit(
        &self,
        repo: &gix::Repository,
        id: gix::ObjectId,
        commit: &gix::Commit<'_>,
        now: i64,
        scores: &mut HashMap<PathBuf, f64>,
        size_cache: &mut HashMap<gix::ObjectId, u64>,
    ) -> Result<()> {
        // Skip merge commits which have more than one parent to keep parity with the original implementation.
        let mut parents = commit.parent_ids();
        let parent = match parents.next() {
            Some(first) if parents.next().is_none() => Some(first.detach()),
            Some(_) => return Ok(()),
            None => None,
        };
        let (from, to) = crate::sdk::diff::commit_trees(repo, id, parent);

        // Obtain a diff object describing changes introduced by this commit.
        let mut changes = crate::sdk::diff::create_changes(&from)?;
        crate::sdk::diff::configure_changes(&mut changes);

        // Older commits contribute less via a quadratic age penalty.
        let weight = self.age_weight(commit, now)?;

        // Apply the weight to all changed paths.
        self.collect_changes(&mut changes, &to, weight, scores, size_cache)
    }

    /// Compute the age-based weight for a commit.
    fn age_weight(&self, commit: &gix::Commit<'_>, now: i64) -> Result<f64> {
        // Calculate time difference in days between the commit and now using integer division
        // so commits less than a day apart yield the same weight. Negative ages are clamped to zero.
        let commit_secs = commit.time().into_diagnostic()?.seconds;
        let age_secs = now.saturating_sub(commit_secs);
        let age_days = (age_secs / SECONDS_PER_DAY) as f64;

        // Younger commits should dominate the score, using a configurable exponent.
        Ok(1.0 / (age_days + 1.0).powf(self.opts.age_exp))
    }

    /// Iterate over all changes of a commit and update scores.
    fn collect_changes(
        &self,
        changes: &mut gix::object::tree::diff::Platform<'_, '_>,
        to: &gix::Tree<'_>,
        weight: f64,
        scores: &mut HashMap<PathBuf, f64>,
        size_cache: &mut HashMap<gix::ObjectId, u64>,
    ) -> Result<()> {
        changes
            .for_each_to_obtain_tree(to, |change| {
                // Skip deletions and non-blob entries; only count file modifications.
                if let Some((path, size)) = self.extract_change_info(change, size_cache) {
                    // Increase score for this path based on change weight and file size.
                    self.update_score(scores, path, size, weight);
                }
                Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
            })
            .into_diagnostic()?;
        Ok(())
    }

    /// Convert a change into a path and blob size if it represents a file.
    fn extract_change_info(
        &self,
        change: gix::object::tree::diff::Change<'_, '_, '_>,
        size_cache: &mut HashMap<gix::ObjectId, u64>,
    ) -> Option<(PathBuf, u64)> {
        use gix::object::tree::diff::Change::*;
        // Extract relevant information depending on the type of change.
        let (mode, id, location) = match change {
            Addition {
                entry_mode,
                id,
                location,
                ..
            }
            | Modification {
                entry_mode,
                id,
                location,
                ..
            } => (entry_mode, id, location),
            Deletion { .. } | Rewrite { .. } => return None,
        };

        // Only blobs (files) contribute to frecency; skip submodules/trees.
        if !mode.is_blob() {
            return None;
        }

        let path = PathBuf::from(location.to_str_lossy().into_owned());

        // Apply the optional path filter provided by the caller.
        if !self.path_allowed(&path) {
            return None;
        }

        // Determine blob size for the penalty term, caching results to avoid
        // repeatedly loading headers in large repositories.
        let size = *size_cache
            .entry(id.detach())
            .or_insert_with(|| id.try_header().ok().flatten().map(|h| h.size()).unwrap_or(0));
        Some((path, size))
    }

    /// Check if the given path is included in the filter list.
    fn path_allowed(&self, path: &PathBuf) -> bool {
        self.opts.paths.as_ref().map_or(true, |paths| paths.contains(path))
    }

    /// Update the score for a single file path.
    fn update_score(&self, scores: &mut HashMap<PathBuf, f64>, path: PathBuf, size: u64, weight: f64) {
        // Smaller files yield higher scores while large blobs are penalized.
        let penalty = self.size_penalty(size);

        // Accumulate weighted score for this path.
        *scores.entry(path).or_default() += penalty * weight;
    }

    /// Return the penalty for a blob of the given size.
    fn size_penalty(&self, size: u64) -> f64 {
        if self.opts.size_ref <= 0.0 {
            1.0
        } else {
            1.0 / (1.0 + ((size as f64) / self.opts.size_ref).sqrt())
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
