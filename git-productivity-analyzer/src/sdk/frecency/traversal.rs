use super::Analyzer;
use crate::error::Result;
use gix::bstr::ByteSlice;
use miette::IntoDiagnostic;
use std::{collections::HashMap, path::PathBuf};

impl Analyzer {
    pub(super) fn collect_scores(
        &self,
        repo: &gix::Repository,
        start: gix::ObjectId,
        since: Option<&gix::ObjectId>,
        now: i64,
    ) -> Result<HashMap<PathBuf, f64>> {
        let mut scores = HashMap::new();
        let mut size_cache = HashMap::new();
        let limit = self.opts.max_commits.unwrap_or(usize::MAX);
        let mut count = 0usize;
        crate::sdk::walk_commits(repo, start, since, |id, commit| {
            if count >= limit {
                return Ok(());
            }
            self.process_commit(repo, id, &commit, now, &mut scores, &mut size_cache)?;
            count += 1;
            Ok(())
        })?;
        Ok(scores)
    }

    pub(super) fn process_commit(
        &self,
        repo: &gix::Repository,
        id: gix::ObjectId,
        commit: &gix::Commit<'_>,
        now: i64,
        scores: &mut HashMap<PathBuf, f64>,
        size_cache: &mut HashMap<gix::ObjectId, u64>,
    ) -> Result<()> {
        let mut parents = commit.parent_ids();
        let parent = match parents.next() {
            Some(first) if parents.next().is_none() => Some(first.detach()),
            Some(_) => return Ok(()),
            None => None,
        };
        let (from, to) = crate::sdk::diff::commit_trees(repo, id, parent);
        let mut changes = crate::sdk::diff::create_changes(&from)?;
        crate::sdk::diff::configure_changes(&mut changes);
        let weight = self.age_weight(commit, now)?;
        self.collect_changes(&mut changes, &to, weight, scores, size_cache)
    }

    pub(super) fn collect_changes(
        &self,
        changes: &mut gix::object::tree::diff::Platform<'_, '_>,
        to: &gix::Tree<'_>,
        weight: f64,
        scores: &mut HashMap<PathBuf, f64>,
        size_cache: &mut HashMap<gix::ObjectId, u64>,
    ) -> Result<()> {
        changes
            .for_each_to_obtain_tree(to, |change| {
                if let Some((path, size)) = self.extract_change_info(change, size_cache) {
                    self.update_score(scores, path, size, weight);
                }
                Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
            })
            .into_diagnostic()?;
        Ok(())
    }

    pub(super) fn extract_change_info(
        &self,
        change: gix::object::tree::diff::Change<'_, '_, '_>,
        size_cache: &mut HashMap<gix::ObjectId, u64>,
    ) -> Option<(PathBuf, u64)> {
        use gix::object::tree::diff::Change::*;
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

        if !mode.is_blob() {
            return None;
        }

        let path = PathBuf::from(location.to_str_lossy().into_owned());
        if !self.path_allowed(&path) {
            return None;
        }

        let size = *size_cache.entry(id.detach()).or_insert_with(|| match id.try_header() {
            Ok(Some(h)) => h.size(),
            Ok(None) => {
                eprintln!("warning: missing header for blob {id}");
                0
            }
            Err(err) => {
                self.log_size_error(id.detach(), &err);
                0
            }
        });
        Some((path, size))
    }
}
