//! Provide analysis of how recently and frequently files changed.
//!
//! Each commit contributes a score to the touched files. The score is the
//! product of a file-size penalty and an age-based weight. Small files changed
//! in recent commits therefore score highest.

// module defining the scoring of files by frequency and recency of changes
// within a repository. The algorithm roughly mirrors the previous
// `frecenfile` crate but is implemented using `gix` for repository access.

use crate::{error::Result, Globals};
use gix::bstr::ByteSlice;
use miette::IntoDiagnostic;
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone)]
pub struct Options {
    pub repo: crate::sdk::RepoOptions,
    pub paths: Option<Vec<PathBuf>>,
    pub max_commits: Option<usize>,
    pub ascending: bool,
    pub descending: bool,
    pub path_only: bool,
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl Analyzer {
    /// Analyze the repository and return each file with its frecency score.
    pub fn analyze(self) -> Result<Vec<(PathBuf, f64)>> {
        // open the repository and resolve the revision range according to
        // global `--since/--until` settings
        let (repo, start, since) = crate::sdk::open_with_range(&self.opts.repo, &self.globals)?;

        // capture the current time once so all age calculations share the same
        // reference point
        let now = Self::current_timestamp();

        // accumulate per-file scores across all relevant commits
        let scores = self.collect_scores(&repo, start, since.as_ref(), now)?;

        // return the scores sorted as requested by the user
        Ok(self.sort_scores(scores))
    }

    /// Walk commits and accumulate scores for changed files.
    fn collect_scores(
        &self,
        repo: &gix::Repository,
        start: gix::ObjectId,
        since: Option<&gix::ObjectId>,
        now: i64,
    ) -> Result<BTreeMap<PathBuf, f64>> {
        // container for accumulating per-path scores
        let mut scores = BTreeMap::new();

        // respect the optional `--max-commits` argument
        let limit = self.opts.max_commits.unwrap_or(usize::MAX);
        let mut count = 0usize;

        // iterate over commits starting at `start` until `since`
        crate::sdk::walk_commits(repo, start, since, |id, commit| {
            // stop walking once the limit is reached
            if count >= limit {
                return Ok(());
            }

            // evaluate all changes introduced by this commit
            self.process_commit(repo, id, &commit, now, &mut scores)?;
            count += 1;
            Ok(())
        })?;

        Ok(scores)
    }

    /// Sort the score map according to user preference.
    fn sort_scores(&self, scores: BTreeMap<PathBuf, f64>) -> Vec<(PathBuf, f64)> {
        // move scores into a vector for sorting
        let mut out: Vec<_> = scores.into_iter().collect();

        // default sort order is descending unless `--ascending` is specified
        let desc = self.opts.descending || !self.opts.ascending;
        out.sort_by(|a, b| {
            // handle NaN gracefully by falling back to `Equal`
            let ord = a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal);
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
        // `SystemTime` is used over chrono to keep dependencies minimal
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn process_commit(
        &self,
        repo: &gix::Repository,
        id: gix::ObjectId,
        commit: &gix::Commit<'_>,
        now: i64,
        scores: &mut BTreeMap<PathBuf, f64>,
    ) -> Result<()> {
        // compare this commit with its first parent (or an empty tree if
        // it has none) to obtain a tree diff
        let parent = commit.parent_ids().next().map(|id| id.detach());
        let (from, to) = crate::sdk::diff::commit_trees(repo, id, parent);

        // prepare the diff configuration so we can iterate over individual
        // file changes
        let mut changes = crate::sdk::diff::create_changes(&from)?;
        crate::sdk::diff::configure_changes(&mut changes);

        // calculate the commit's time-based weight once
        let weight = self.age_weight(commit, now)?;

        // finally collect all file level changes and update their scores
        self.collect_changes(&mut changes, &to, weight, scores)
    }

    /// Compute the age-based weight for a commit.
    fn age_weight(&self, commit: &gix::Commit<'_>, now: i64) -> Result<f64> {
        // convert the difference between now and the commit timestamp into days
        let age_days = (now - commit.time().into_diagnostic()?.seconds) as f64 / 86_400.0;

        // weight commits quadratically - the older the commit the smaller its
        // contribution becomes
        Ok(1.0 / (age_days + 1.0).powi(2))
    }

    /// Iterate over all changes of a commit and update scores.
    fn collect_changes(
        &self,
        changes: &mut gix::object::tree::diff::Platform<'_, '_>,
        to: &gix::Tree<'_>,
        weight: f64,
        scores: &mut BTreeMap<PathBuf, f64>,
    ) -> Result<()> {
        changes
            .for_each_to_obtain_tree(to, |change| {
                // translate the diff entry into a filesystem path and blob size
                if let Some((path, size)) = self.extract_change_info(change) {
                    // apply the weight and size penalty to update the running total
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

        // only additions and modifications contribute to frecency
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

        // skip non-file entries such as submodules or trees
        if !mode.is_blob() {
            return None;
        }

        // convert the change location into a real path
        let path = PathBuf::from(location.to_str_lossy().into_owned());

        // respect optional user-provided path filtering
        if !self.path_allowed(&path) {
            return None;
        }

        // look up the blob to obtain its size; missing blobs default to 0
        let size = id.object().map(|b| b.data.len() as u64).unwrap_or(0);

        Some((path, size))
    }

    /// Check if the given path is included in the filter list.
    fn path_allowed(&self, path: &PathBuf) -> bool {
        // when no filter is supplied all paths are considered
        self.opts.paths.as_ref().map_or(true, |paths| paths.contains(path))
    }

    /// Update the score for a single file path.
    fn update_score(&self, scores: &mut BTreeMap<PathBuf, f64>, path: PathBuf, size: u64, weight: f64) {
        // penalize large files so tiny files changed frequently rank higher
        let penalty = Self::size_penalty(size);
        *scores.entry(path).or_default() += penalty * weight;
    }

    /// Return the penalty for a blob of the given size.
    fn size_penalty(size: u64) -> f64 {
        // a 1KB blob receives full score while larger blobs diminish
        1.0 / (1.0 + ((size as f64) / 1024.0).sqrt())
    }

    pub fn print_scores(&self, scores: &[(PathBuf, f64)]) {
        crate::sdk::print_json_or(
            self.globals.json,
            &SerializableScores::from(scores, self.opts.path_only),
            || {
                for (path, score) in scores {
                    if self.opts.path_only {
                        // only show the file path if requested
                        println!("{}", path.display());
                    } else {
                        // default output prints the score followed by the path
                        println!("{:.4}\t{}", score, path.display());
                    }
                }
            },
        );
    }
}

#[derive(Serialize)]
/// Helper type to shape JSON output of the analyzer.
struct SerializableScores {
    scores: Vec<(String, f64)>,
}

impl SerializableScores {
    fn from(list: &[(PathBuf, f64)], path_only: bool) -> Self {
        Self {
            scores: list
                .iter()
                .map(|(p, s)| {
                    // hide the score value entirely in path-only mode to keep
                    // JSON shape consistent with the plain text output
                    if path_only {
                        (p.display().to_string(), 0.0)
                    } else {
                        (p.display().to_string(), *s)
                    }
                })
                .collect(),
        }
    }
}

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Vec<(PathBuf, f64)>;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
