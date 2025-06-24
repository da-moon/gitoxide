use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The directory containing a '.git/' folder.
    #[arg(long = "working-dir", default_value = ".")]
    pub working_dir: PathBuf,

    /// The name of the revision as spec at which to start iterating the commit graph.
    #[arg(long = "rev-spec", default_value = "HEAD")]
    pub rev_spec: String,

    /// Ignore github bots which match the `[bot]` search string.
    #[arg(long = "no-bots", short = 'b')]
    pub no_bots: bool,

    /// Collect additional information about file modifications, additions and deletions.
    #[arg(long = "file-stats", short = 'f')]
    pub file_stats: bool,

    /// Collect additional information about lines added and deleted.
    #[arg(long = "line-stats", short = 'l')]
    pub line_stats: bool,

    /// Show personally identifiable information before the summary. Includes names and email addresses.
    #[arg(long = "show-pii", short = 'p')]
    pub show_pii: bool,

    /// Omit unifying identities by name and email which can lead to the same author appearing multiple times.
    #[arg(long = "omit-unify-identities", short = 'i')]
    pub omit_unify_identities: bool,

    /// The amount of threads to use. If unset, use all cores, if 0 use all physical cores.
    #[arg(long, short = 't')]
    pub threads: Option<usize>,
}
