//! CLI argument definition for the `frecency` command.

use crate::cmd::common::CommonArgs;
use clap::{Args as ClapArgs, ValueHint};
use std::path::PathBuf;

/// Parameters accepted by the `frecency` subcommand.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(long, value_hint = ValueHint::FilePath, num_args = 1.., help = "Only include these paths")]
    pub paths: Vec<PathBuf>,

    #[arg(long, help = "Limit to the newest N commits")]
    pub max_commits: Option<usize>,

    #[arg(long, conflicts_with = "descending", help = "Sort scores ascending")]
    pub ascending: bool,

    #[arg(long, conflicts_with = "ascending", help = "Sort scores descending")]
    pub descending: bool,

    #[arg(long, help = "Only print file paths")]
    pub path_only: bool,
}

impl From<Args> for crate::sdk::frecency::Options {
    fn from(a: Args) -> Self {
        // `None` indicates no filtering; `Vec` stores user provided paths
        let paths = if a.paths.is_empty() { None } else { Some(a.paths) };
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
