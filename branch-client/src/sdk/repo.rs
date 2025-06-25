//! Small helper for opening a repository using [`gix`].

use miette::IntoDiagnostic;
use std::path::Path;

use crate::error::Result;

/// Open the repository located at `path`.
///
/// This simply calls [`gix::open`] and converts the error into our
/// [`miette`] based [`crate::error::Result`] type.
pub fn open_repo(path: &Path) -> Result<gix::Repository> {
    gix::open(path).into_diagnostic()
}
