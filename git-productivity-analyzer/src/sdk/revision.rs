use crate::error::Result;
use gix::bstr::ByteSlice;
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
