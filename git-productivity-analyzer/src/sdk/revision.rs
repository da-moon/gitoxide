use crate::error::Result;
use gix::{bstr::ByteSlice, prelude::*};
use miette::IntoDiagnostic;

pub fn resolve_start_commit(repo: &gix::Repository, rev_spec: &str, until: Option<&str>) -> Result<gix::ObjectId> {
    let spec = until.unwrap_or(rev_spec);
    Ok(repo
        .rev_parse_single(spec.as_bytes().as_bstr())
        .into_diagnostic()?
        .detach())
}

pub fn resolve_since_commit(repo: &gix::Repository, since: Option<&str>) -> Result<Option<gix::ObjectId>> {
    match since {
        Some(spec) => Ok(Some(
            repo.rev_parse_single(spec.as_bytes().as_bstr())
                .into_diagnostic()?
                .detach(),
        )),
        None => Ok(None),
    }
}

/// Walk commits starting at `start` until `since` is reached.
///
/// The commit identified by `since`, if provided, is **included** in the
/// iteration before stopping.
pub fn walk_commits<'repo, F>(
    repo: &'repo gix::Repository,
    start: gix::ObjectId,
    since: Option<&gix::ObjectId>,
    mut f: F,
) -> Result<()>
where
    F: FnMut(gix::ObjectId, gix::Commit<'repo>) -> Result<()>,
{
    for item in start.ancestors(&repo.objects) {
        let info = item.into_diagnostic()?;
        {
            let commit = repo.find_commit(info.id).into_diagnostic()?;
            f(info.id, commit)?;
        }
        if let Some(id) = since {
            if &info.id == id {
                break;
            }
        }
    }
    Ok(())
}

pub fn open_with_range(
    opts: &crate::sdk::RepoOptions,
    globals: &crate::Globals,
) -> Result<(gix::Repository, gix::ObjectId, Option<gix::ObjectId>)> {
    let repo = gix::discover(&opts.working_dir).into_diagnostic()?;
    let start = resolve_start_commit(&repo, &opts.rev_spec, globals.until.as_deref())?;
    let since = resolve_since_commit(&repo, globals.since.as_deref())?;
    Ok((repo, start, since))
}
