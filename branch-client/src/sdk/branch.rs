//! Helpers for common branch operations built on top of [`gix`].
//!
//! The functions in this module provide minimal wrappers around `gix` APIs so
//! each CLI command can reuse them. They return [`crate::error::Result`] for a
//! consistent error type.

use miette::IntoDiagnostic;

use crate::error::Result;

/// Return the list of local or remote branch names.
pub fn list_branches(repo: &gix::Repository, remote: bool) -> Result<Vec<String>> {
    // Obtain an iterator over all references in the repository.
    let references = repo.references().into_diagnostic()?;
    // Depending on the flag, narrow the iterator to either local or remote
    // branches. `remote_branches` and `local_branches` filter the reference
    // namespace for `refs/remotes/*` and `refs/heads/*` respectively.
    let refs = if remote {
        references.remote_branches().into_diagnostic()?
    } else {
        references.local_branches().into_diagnostic()?
    };
    let mut names = Vec::new();
    // Each item of `refs` yields a result with a `Reference`. Convert the
    // reference into a printable short name like `main` and collect.
    for r in refs {
        let r = r.map_err(|e| miette::Report::msg(e.to_string()))?;
        names.push(r.name().shorten().to_string());
    }
    Ok(names)
}

/// Create a new branch `name` starting from revision `start`.
pub fn create_branch(repo: &gix::Repository, name: &str, start: &str) -> Result<()> {
    // `rev_parse_single` resolves the provided revision string (like "HEAD" or a
    // commit hash) to an object id.
    let id = repo.rev_parse_single(start).into_diagnostic()?;
    // Create `refs/heads/<name>` pointing to that id. `PreviousValue::MustNotExist`
    // ensures the reference did not already exist.
    repo.reference(
        format!("refs/heads/{name}"),
        id,
        gix::refs::transaction::PreviousValue::MustNotExist,
        "branch created",
    )
    .into_diagnostic()?;
    Ok(())
}

/// Remove the branch `name` from `refs/heads/`.
pub fn delete_branch(repo: &gix::Repository, name: &str) -> Result<()> {
    // Look up the reference and delete it in one call.
    let r = repo.find_reference(&format!("refs/heads/{name}")).into_diagnostic()?;
    r.delete().into_diagnostic()?;
    Ok(())
}

/// Return `(ahead, behind)` commit counts comparing `lhs` and `rhs`.
pub fn compare_branches(repo: &gix::Repository, lhs: &str, rhs: &str) -> Result<(usize, usize)> {
    // Resolve both revision strings to object ids.
    let lhs = repo.rev_parse_single(lhs).into_diagnostic()?;
    let rhs = repo.rev_parse_single(rhs).into_diagnostic()?;

    // Count commits reachable from `lhs` but not `rhs`.
    // `rev_walk` starts an iterator over commits and `with_boundary` stops once
    // the boundary revision is encountered.
    let ahead = repo
        .rev_walk([lhs])
        .with_boundary([rhs])
        .all()
        .into_diagnostic()?
        .filter_map(std::result::Result::ok)
        .count();

    // Count commits reachable from `rhs` but not `lhs` in the same fashion.
    let behind = repo
        .rev_walk([rhs])
        .with_boundary([lhs])
        .all()
        .into_diagnostic()?
        .filter_map(std::result::Result::ok)
        .count();

    Ok((ahead, behind))
}

/// Delete or list local branches fully merged into `HEAD`.
pub fn cleanup_merged_branches(repo: &gix::Repository, dry_run: bool) -> Result<Vec<String>> {
    // Determine the commit id of HEAD. Branches pointing to this id are skipped.
    let head = repo.head_id().into_diagnostic()?;
    let mut removed = Vec::new();

    // Iterate over all local branches.
    for r in repo
        .references()
        .into_diagnostic()?
        .local_branches()
        .into_diagnostic()?
    {
        let mut r = r.map_err(|e| miette::Report::msg(e.to_string()))?;
        let name = r.name().shorten().to_string();
        // Obtain the object id the branch points to.
        let id = r.peel_to_id_in_place().into_diagnostic()?;

        // Skip the branch if it's the current HEAD.
        if id == head {
            continue;
        }

        // `merge_base` returns the common ancestor of the two commits. If the
        // branch tip equals the merge base with HEAD, then it is fully merged.
        if repo.merge_base(head, id).map(|b| b == id).unwrap_or(false) {
            if dry_run {
                // Only report that it would be removed.
                removed.push(name);
            } else {
                // Actually delete the reference and record its name.
                r.delete().into_diagnostic()?;
                removed.push(name);
            }
        }
    }

    Ok(removed)
}
