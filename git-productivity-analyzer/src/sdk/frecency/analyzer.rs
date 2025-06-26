//! Provide analysis of how recently and frequently files changed.
//!
//! Each commit contributes a score to the touched files. The score is the
//! product of a file-size penalty and an age-based weight. Small files changed
//! in recent commits therefore score highest.

use crate::{error::Result, Globals};
use clap::ValueEnum;
use gix::bstr::ByteSlice;
use miette::IntoDiagnostic;
use serde::Serialize;
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
const SECONDS_PER_DAY: f64 = 86_400.0;
/// Exponent applied to the age weight for quadratic decay.
const AGE_EXPONENT: i32 = 2;
/// Reference size in bytes for the size penalty computation.
const SIZE_PENALTY_REF: f64 = 1024.0;

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
}

#[derive(Clone)]
/// Analyze git history to compute file frecency scores.
pub struct Analyzer {
    /// Options provided by the caller.
    opts: Options,
    /// Global flags controlling repository access and output.
    globals: Globals,
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl Analyzer {
    /// Analyze the repository and return each file with its frecency score.
    pub fn analyze(self) -> Result<Vec<(PathBuf, f64)>> {
        // Open the repository and resolve the commit range specified via global flags.
        // If no commits exist yet, return an empty result instead of an error.
        let (repo, start, since) = match crate::sdk::open_with_range(&self.opts.repo, &self.globals) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };

        // Use the current time to weight commits based on their age.
        let now = Self::current_timestamp();

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

        // Optional commit limit to reduce execution time on large repositories.
        let limit = self.opts.max_commits.unwrap_or(usize::MAX);
        let mut count = 0usize;
        crate::sdk::walk_commits(repo, start, since, |id, commit| {
            // Stop iteration once the desired number of commits has been seen.
            if count >= limit {
                return Ok(());
            }

            // Accumulate frecency contributions from this commit.
            self.process_commit(repo, id, &commit, now, &mut scores)?;
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
            let ord = a.1.total_cmp(&b.1);
            if desc {
                ord.reverse()
            } else {
                ord
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
    ) -> Result<()> {
        // Determine the parent tree to diff against; the first parent if it exists.
        let parent = commit.parent_ids().next().map(|id| id.detach());
        let (from, to) = crate::sdk::diff::commit_trees(repo, id, parent);

        // Obtain a diff object describing changes introduced by this commit.
        let mut changes = crate::sdk::diff::create_changes(&from)?;
        crate::sdk::diff::configure_changes(&mut changes);

        // Older commits contribute less via a quadratic age penalty.
        let weight = self.age_weight(commit, now)?;

        // Apply the weight to all changed paths.
        self.collect_changes(&mut changes, &to, weight, scores)
    }

    /// Compute the age-based weight for a commit.
    fn age_weight(&self, commit: &gix::Commit<'_>, now: i64) -> Result<f64> {
        // Calculate time difference in days between the commit and now.
        let mut age_days = (now - commit.time().into_diagnostic()?.seconds) as f64 / SECONDS_PER_DAY;
        if age_days < 0.0 {
            age_days = 0.0;
        }

        // Younger commits should dominate the score, hence a squared decay.
        Ok(1.0 / (age_days + 1.0).powi(AGE_EXPONENT))
    }

    /// Iterate over all changes of a commit and update scores.
    fn collect_changes(
        &self,
        changes: &mut gix::object::tree::diff::Platform<'_, '_>,
        to: &gix::Tree<'_>,
        weight: f64,
        scores: &mut HashMap<PathBuf, f64>,
    ) -> Result<()> {
        changes
            .for_each_to_obtain_tree(to, |change| {
                // Skip deletions and non-blob entries; only count file modifications.
                if let Some((path, size)) = self.extract_change_info(change) {
                    // Increase score for this path based on change weight and file size.
                    self.update_score(scores, path, size, weight);
                }
                Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
            })
            .into_diagnostic()?;
        Ok(())
    }

    /// Convert a change into a path and blob size if it represents a file.
    fn extract_change_info(&self, change: gix::object::tree::diff::Change<'_, '_, '_>) -> Option<(PathBuf, u64)> {
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

        // Determine blob size for the penalty term without loading the entire blob.
        let size = id.try_header().ok().flatten().map(|h| h.size()).unwrap_or(0);
        Some((path, size))
    }

    /// Check if the given path is included in the filter list.
    fn path_allowed(&self, path: &PathBuf) -> bool {
        self.opts.paths.as_ref().map_or(true, |paths| paths.contains(path))
    }

    /// Update the score for a single file path.
    fn update_score(&self, scores: &mut HashMap<PathBuf, f64>, path: PathBuf, size: u64, weight: f64) {
        // Smaller files yield higher scores while large blobs are penalized.
        let penalty = Self::size_penalty(size);

        // Accumulate weighted score for this path.
        *scores.entry(path).or_default() += penalty * weight;
    }

    /// Return the penalty for a blob of the given size.
    fn size_penalty(size: u64) -> f64 {
        1.0 / (1.0 + ((size as f64) / SIZE_PENALTY_REF).sqrt())
    }

    /// Print the computed scores either as text table or JSON.
    pub fn print_scores(&self, scores: &[(PathBuf, f64)]) {
        if self.opts.path_only {
            crate::sdk::print_json_or(self.globals.json, &SerializablePaths::from(scores), || {
                for (path, _) in scores {
                    println!("{}", path.display());
                }
            });
        } else {
            crate::sdk::print_json_or(self.globals.json, &SerializableScores::from(scores), || {
                for (path, score) in scores {
                    println!("{:.4}\t{}", score, path.display());
                }
            });
        }
    }
}

#[derive(Serialize)]
/// Helper struct to make printing and JSON serialization uniform.
struct SerializableScores {
    scores: Vec<(String, f64)>,
}

impl SerializableScores {
    /// Convert `(PathBuf, f64)` tuples into owned strings for JSON output.
    fn from(list: &[(PathBuf, f64)]) -> Self {
        Self {
            scores: list.iter().map(|(p, s)| (p.display().to_string(), *s)).collect(),
        }
    }
}

#[derive(Serialize)]
/// Simple JSON representation for path-only output.
struct SerializablePaths {
    paths: Vec<String>,
}

impl SerializablePaths {
    /// Convert the sorted list of paths into owned strings for JSON output.
    fn from(list: &[(PathBuf, f64)]) -> Self {
        Self {
            paths: list.iter().map(|(p, _)| p.display().to_string()).collect(),
        }
    }
}

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Vec<(PathBuf, f64)>;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
