use std::collections::BTreeMap;
use std::path::PathBuf;

use super::diff_utils::{compute_diff_lines, configure_changes, create_changes};
use crate::{error::Result, Globals};
use gix::{
    bstr::{BStr, ByteSlice},
    prelude::*,
};
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
    pub working_dir: PathBuf,
    pub rev_spec: String,
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
        id.object()
            .map(|blob| blob.data.as_slice().lines_with_terminator().count() as u32)
            .unwrap_or(0)
    }

    pub fn analyze(self) -> Result<Summary> {
        let repo = gix::discover(&self.opts.working_dir).into_diagnostic()?;
        let start = crate::sdk::resolve_start_commit(&repo, &self.opts.rev_spec, self.globals.until.as_deref())?;
        let since = crate::sdk::resolve_since_commit(&repo, self.globals.since.as_deref())?;
        let mut totals = BTreeMap::<String, Counts>::new();
        self.walk_commits(&repo, start, since.as_ref(), &mut totals)?;
        Ok(Summary { totals })
    }

    fn walk_commits(
        &self,
        repo: &gix::Repository,
        start: gix::ObjectId,
        since: Option<&gix::ObjectId>,
        totals: &mut BTreeMap<String, Counts>,
    ) -> Result<()> {
        let mut buf = Vec::new();
        let mut cache = repo
            .diff_resource_cache(gix::diff::blob::pipeline::Mode::ToGit, Default::default())
            .map_err(|e| miette::Report::msg(e.to_string()))?;
        for item in start.ancestors(&repo.objects) {
            let info = item.into_diagnostic()?;
            let commit = repo.objects.find_commit_iter(&info.id, &mut buf).into_diagnostic()?;
            let author = commit.author().into_diagnostic()?;
            if let Some(pattern) = &self.opts.author {
                let pat = pattern.as_str();
                if !author.name.to_str_lossy().contains(pat) && !author.email.to_str_lossy().contains(pat) {
                    if let Some(id) = since {
                        if &info.id == id {
                            break;
                        }
                    }
                    continue;
                }
            }
            let author_string = format!("{} <{}>", author.name, author.email);
            self.process_commit(repo, &mut cache, info.id, &commit, &author_string, totals)?;
            if let Some(id) = since {
                if &info.id == id {
                    break;
                }
            }
        }
        Ok(())
    }

    fn process_commit(
        &self,
        repo: &gix::Repository,
        cache: &mut gix::diff::blob::Platform,
        commit_id: gix::ObjectId,
        commit: &gix::objs::CommitRefIter<'_>,
        author: &str,
        totals: &mut BTreeMap<String, Counts>,
    ) -> Result<()> {
        let (from, to) = self.trees(repo, commit_id, commit.parent_ids().next());
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

    fn trees<'repo>(
        &self,
        repo: &'repo gix::Repository,
        commit_id: gix::ObjectId,
        parent: Option<gix::ObjectId>,
    ) -> (gix::Tree<'repo>, gix::Tree<'repo>) {
        let to = repo
            .find_object(commit_id)
            .ok()
            .and_then(|o| o.peel_to_tree().ok())
            .unwrap_or_else(|| repo.empty_tree());
        let from = parent
            .and_then(|id| repo.find_object(id).ok())
            .and_then(|c| c.peel_to_tree().ok())
            .unwrap_or_else(|| repo.empty_tree());
        (from, to)
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
