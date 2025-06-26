use gix::bstr::BStr;
use gix::object::tree::diff::Change;

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
