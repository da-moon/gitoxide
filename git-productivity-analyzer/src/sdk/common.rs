use std::path::PathBuf;

/// Options shared across all analyzers.
#[derive(Clone)]
pub struct RepoOptions {
    pub working_dir: PathBuf,
    pub rev_spec: String,
}

impl From<crate::cmd::common::CommonArgs> for RepoOptions {
    fn from(args: crate::cmd::common::CommonArgs) -> Self {
        Self {
            working_dir: args.working_dir,
            rev_spec: args.rev_spec,
        }
    }
}
