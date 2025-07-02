pub mod churn;
pub mod commit_frequency;
pub mod commit_size;
mod common;
pub mod diff;
pub mod frecency;
mod helpers;
pub mod hours;
mod revision;
pub mod stats;
pub mod time_of_day;

pub use helpers::{print_json_or, run_with_analyzer, AnalyzerTrait, IntoAnalyzer};

pub use common::RepoOptions;

pub use helpers::author_matches;
#[allow(unused_imports)]
pub use revision::{open_with_range, resolve_since_commit, resolve_start_commit, walk_commits};
