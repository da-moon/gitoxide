use crate::error::Result;
use gix::{
    bstr::BStr,
    object::tree::diff::{Change, Platform},
    Tree,
};
use miette::Report;

pub fn create_changes<'a, 'repo>(tree: &'a Tree<'repo>) -> Result<Platform<'a, 'repo>> {
    tree.changes().map_err(|e| Report::msg(e.to_string()))
}

pub fn configure_changes(changes: &mut Platform<'_, '_>) {
    changes.options(|opts| {
        opts.track_filename().track_path().track_rewrites(None);
    });
}

pub fn compute_diff_lines(
    prev_mode: gix::object::tree::EntryMode,
    new_mode: gix::object::tree::EntryMode,
    prev_id: gix::Id<'_>,
    new_id: gix::Id<'_>,
    location: &BStr,
    cache: &mut gix::diff::blob::Platform,
) -> Option<(u32, u32)> {
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
