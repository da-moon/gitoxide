use std::collections::BTreeMap;

use super::diff_utils::compute_diff_lines;
use crate::sdk::diff::{commit_trees, configure_changes, create_changes};
use crate::{error::Result, Globals};
use bytecount::count;
use gix::bstr::{BStr, ByteSlice};
use miette::IntoDiagnostic;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Counts {
    pub added: u32,
    pub removed: u32,
}

#[derive(Serialize)]
pub struct Summary {
    pub totals: BTreeMap<String, Counts>,
}

#[derive(Clone)]
pub struct Options {
    pub repo: crate::sdk::RepoOptions,
    pub per_file: bool,
    pub author: Option<String>,
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

impl Analyzer {
    fn key(&self, author: &str, location: &BStr) -> String {
        if self.opts.per_file {
            location.to_str_lossy().into_owned()
        } else {
            author.to_string()
        }
    }

    fn add_total(totals: &mut BTreeMap<String, Counts>, key: String, added: u32, removed: u32) {
        let entry = totals.entry(key).or_default();
        entry.added += added;
        entry.removed += removed;
    }

    fn count_lines(id: gix::Id<'_>) -> u32 {
        fn count_lines_in_bytes(data: &[u8]) -> u32 {
            if data.is_empty() {
                return 0;
            }
            let mut count = count(data, b'\n') as u32;
            if data.last() != Some(&b'\n') {
                count += 1;
            }
            count
        }

        id.object()
            .map(|blob| count_lines_in_bytes(blob.data.as_slice()))
            .unwrap_or(0)
    }

    pub fn analyze(self) -> Result<Summary> {
        let (repo, start, since) = crate::sdk::open_with_range(&self.opts.repo, &self.globals)?;
        let mut totals = BTreeMap::<String, Counts>::new();
        let mut cache = repo
            .diff_resource_cache(gix::diff::blob::pipeline::Mode::ToGit, Default::default())
            .map_err(|e| miette::Report::msg(e.to_string()))?;
        crate::sdk::walk_commits(&repo, start, since.as_ref(), |id, commit| {
            let author = commit.author().into_diagnostic()?;
            if !crate::sdk::author_matches(&author, &self.opts.author) {
                return Ok(());
            }
            let author_string = format!("{} <{}>", author.name, author.email);
            self.process_commit(&repo, &mut cache, id, &commit, &author_string, &mut totals)
        })?;
        Ok(Summary { totals })
    }

    fn process_commit(
        &self,
        repo: &gix::Repository,
        cache: &mut gix::diff::blob::Platform,
        commit_id: gix::ObjectId,
        commit: &gix::Commit<'_>,
        author: &str,
        totals: &mut BTreeMap<String, Counts>,
    ) -> Result<()> {
        let parent = commit.parent_ids().next().map(|id| id.detach());
        let (from, to) = commit_trees(repo, commit_id, parent);
        let mut diff = create_changes(&from)?;
        configure_changes(&mut diff);
        self.process_changes(&mut diff, &to, author, totals, cache)
    }

    fn process_changes(
        &self,
        changes: &mut gix::object::tree::diff::Platform<'_, '_>,
        to: &gix::Tree<'_>,
        author: &str,
        totals: &mut BTreeMap<String, Counts>,
        cache: &mut gix::diff::blob::Platform,
    ) -> Result<()> {
        changes
            .for_each_to_obtain_tree(to, |change| {
                self.apply_change(change, author, totals, cache);
                Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
            })
            .map_err(|e| miette::Report::msg(e.to_string()))?;
        Ok(())
    }

    fn apply_change(
        &self,
        change: gix::object::tree::diff::Change<'_, '_, '_>,
        author: &str,
        totals: &mut BTreeMap<String, Counts>,
        cache: &mut gix::diff::blob::Platform,
    ) {
        use gix::object::tree::diff::Change::*;
        match change {
            Addition {
                entry_mode,
                id,
                location,
                ..
            } => self.apply_addition(entry_mode, id, location, author, totals),
            Deletion {
                entry_mode,
                id,
                location,
                ..
            } => self.apply_deletion(entry_mode, id, location, author, totals),
            Modification {
                entry_mode,
                previous_entry_mode,
                id,
                previous_id,
                location,
                ..
            } => self.handle_modification(
                previous_entry_mode,
                entry_mode,
                previous_id,
                id,
                location,
                author,
                totals,
                cache,
            ),
            Rewrite { .. } => {}
        }
    }

    fn apply_addition(
        &self,
        mode: gix::object::tree::EntryMode,
        id: gix::Id<'_>,
        location: &BStr,
        author: &str,
        totals: &mut BTreeMap<String, Counts>,
    ) {
        if mode.is_blob() {
            let lines = Self::count_lines(id);
            let key = self.key(author, location);
            Self::add_total(totals, key, lines, 0);
        }
    }

    fn apply_deletion(
        &self,
        mode: gix::object::tree::EntryMode,
        id: gix::Id<'_>,
        location: &BStr,
        author: &str,
        totals: &mut BTreeMap<String, Counts>,
    ) {
        if mode.is_blob() {
            let lines = Self::count_lines(id);
            let key = self.key(author, location);
            Self::add_total(totals, key, 0, lines);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_modification(
        &self,
        prev_mode: gix::object::tree::EntryMode,
        new_mode: gix::object::tree::EntryMode,
        prev_id: gix::Id<'_>,
        new_id: gix::Id<'_>,
        location: &BStr,
        author: &str,
        totals: &mut BTreeMap<String, Counts>,
        cache: &mut gix::diff::blob::Platform,
    ) {
        match (prev_mode.is_blob(), new_mode.is_blob()) {
            (false, false) => {}
            (false, true) => self.add_blob_lines(new_id, location, author, totals),
            (true, false) => self.remove_blob_lines(prev_id, location, author, totals),
            (true, true) => {
                if let Some((added, removed)) =
                    compute_diff_lines(prev_mode, new_mode, prev_id, new_id, location, cache)
                {
                    let key = self.key(author, location);
                    Self::add_total(totals, key, added, removed);
                }
            }
        }
    }

    fn add_blob_lines(&self, id: gix::Id<'_>, location: &BStr, author: &str, totals: &mut BTreeMap<String, Counts>) {
        let lines = Self::count_lines(id);
        let key = self.key(author, location);
        Self::add_total(totals, key, lines, 0);
    }

    fn remove_blob_lines(&self, id: gix::Id<'_>, location: &BStr, author: &str, totals: &mut BTreeMap<String, Counts>) {
        let lines = Self::count_lines(id);
        let key = self.key(author, location);
        Self::add_total(totals, key, 0, lines);
    }

    pub fn print_summary(&self, summary: &Summary) {
        crate::sdk::print_json_or(self.globals.json, summary, || {
            for (key, counts) in &summary.totals {
                println!("{key}: +{} -{}", counts.added, counts.removed);
            }
        });
    }
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Summary;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
