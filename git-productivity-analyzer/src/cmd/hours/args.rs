use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(
        long = "working-dir",
        default_value = ".",
        help = "The directory containing a '.git/' folder."
    )]
    pub working_dir: PathBuf,

    #[arg(
        long = "rev-spec",
        default_value = "HEAD",
        help = "The name of the revision as spec at which to start iterating the commit graph."
    )]
    pub rev_spec: String,

    #[arg(
        long = "no-bots",
        short = 'b',
        help = "Ignore github bots which match the `[bot]` search string."
    )]
    pub no_bots: bool,

    #[arg(
        long = "file-stats",
        short = 'f',
        help = "Collect additional information about file modifications, additions and deletions."
    )]
    pub file_stats: bool,

    #[arg(
        long = "line-stats",
        short = 'l',
        help = "Collect additional information about lines added and deleted."
    )]
    pub line_stats: bool,

    #[arg(
        long = "show-pii",
        short = 'p',
        help = "Show personally identifiable information before the summary. Includes names and email addresses."
    )]
    pub show_pii: bool,

    #[arg(
        long = "omit-unify-identities",
        short = 'i',
        help = "Omit unifying identities by name and email which can lead to the same author appearing multiple times."
    )]
    pub omit_unify_identities: bool,

    #[arg(
        long,
        short = 't',
        help = "The amount of threads to use. If unset, use all cores, if 0 use all physical cores."
    )]
    pub threads: Option<usize>,
}
