use crate::error::Result;
use gix::{object::tree::diff::Platform, Tree};
use miette::IntoDiagnostic;

/// Create a diff platform for the given tree.
pub fn create_changes<'a, 'repo>(tree: &'a Tree<'repo>) -> Result<Platform<'a, 'repo>> {
    tree.changes().into_diagnostic()
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

/// Compute added and removed line counts between two blobs.
pub fn compute_diff_lines(
    prev_mode: gix::object::tree::EntryMode,
    new_mode: gix::object::tree::EntryMode,
    prev_id: gix::Id<'_>,
    new_id: gix::Id<'_>,
    location: &gix::bstr::BStr,
    cache: &mut gix::diff::blob::Platform,
) -> Option<(u32, u32)> {
    use gix::object::tree::diff::Change;
    let change = Change::Modification {
        entry_mode: new_mode,
        previous_entry_mode: prev_mode,
        id: new_id,
        previous_id: prev_id,
        location,
    };
    if let Ok(mut diff) = change.diff(cache) {
        diff.line_counts()
            .ok()
            .flatten()
            .map(|counts| (counts.insertions, counts.removals))
    } else {
        None
    }
}
