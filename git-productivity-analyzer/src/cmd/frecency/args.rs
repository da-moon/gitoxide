use crate::cmd::common::CommonArgs;
use clap::{Args as ClapArgs, ValueHint};
use std::{collections::HashSet, path::PathBuf};

#[derive(Debug, ClapArgs)]
/// Command line flags accepted by the `frecency` subcommand.
pub struct Args {
    /// Shared options specifying repository location and revision range.
    #[command(flatten)]
    pub common: CommonArgs,

    /// Optional list of paths to include. When empty all paths are analyzed.
    #[arg(long, value_hint = ValueHint::FilePath, num_args = 1.., help = "Only include these paths")]
    pub paths: Vec<PathBuf>,

    /// Limit the analysis to the newest `n` commits.
    #[arg(long, help = "Limit to the newest N commits")]
    pub max_commits: Option<usize>,

    /// Sort results from lowest to highest score.
    #[arg(long, conflicts_with = "descending", help = "Sort scores ascending")]
    pub ascending: bool,

    /// Sort results from highest to lowest score.
    #[arg(long, conflicts_with = "ascending", help = "Sort scores descending")]
    pub descending: bool,

    /// Print only the path for each entry, omitting the score column.
    #[arg(long, help = "Only print file paths")]
    pub path_only: bool,
}

impl From<Args> for crate::sdk::frecency::Options {
    /// Convert CLI arguments into SDK options, normalizing empty path lists.
    fn from(a: Args) -> Self {
        let paths = if a.paths.is_empty() {
            None
        } else {
            Some(a.paths.into_iter().collect::<HashSet<_>>())
        };
        Self {
            repo: a.common.into(),
            paths,
            max_commits: a.max_commits,
            ascending: a.ascending,
            descending: a.descending,
            path_only: a.path_only,
        }
    }
}
