use super::{analyzer::SECONDS_PER_DAY, Analyzer};
use crate::error::Result;
use miette::IntoDiagnostic;
use std::{collections::HashMap, path::PathBuf};

impl Analyzer {
    /// Compute the age-based weight for a commit.
    pub(super) fn age_weight(&self, commit: &gix::Commit<'_>, now: i64) -> Result<f64> {
        let commit_secs = commit.time().into_diagnostic()?.seconds;
        let age_secs = now.saturating_sub(commit_secs);
        let age_days = (age_secs / SECONDS_PER_DAY) as f64;
        Ok(1.0 / (age_days + 1.0).powf(self.opts.age_exp))
    }

    /// Check if the given path is included in the filter list.
    pub(super) fn path_allowed(&self, path: &PathBuf) -> bool {
        self.opts.paths.as_ref().map_or(true, |paths| paths.contains(path))
    }

    /// Update the score for a single file path.
    pub(super) fn update_score(&self, scores: &mut HashMap<PathBuf, f64>, path: PathBuf, size: u64, weight: f64) {
        let penalty = self.size_penalty(size);
        *scores.entry(path).or_default() += penalty * weight;
    }

    /// Return the penalty for a blob of the given size.
    pub(super) fn size_penalty(&self, size: u64) -> f64 {
        1.0 / (1.0 + ((size as f64) / self.opts.size_ref).sqrt())
    }

    pub(super) fn log_size_error(&self, id: gix::ObjectId, err: &dyn std::error::Error) {
        log::warn!("failed to read header for blob {id}: {err}");
    }
}
