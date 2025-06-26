use crate::error::Result;
use gix::{object::tree::diff::Platform, Tree};
use miette::Report;

/// Create a diff platform for the given tree.
pub fn create_changes<'a, 'repo>(tree: &'a Tree<'repo>) -> Result<Platform<'a, 'repo>> {
    tree.changes().map_err(|e| Report::msg(e.to_string()))
}

/// Configure the diff platform with rename tracking and path tracking.
pub fn configure_changes(changes: &mut Platform<'_, '_>) {
    changes.options(|opts| {
        opts.track_filename().track_path().track_rewrites(None);
    });
}

/// Return the trees of a commit and its first parent if present.
pub fn commit_trees<'repo>(
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
