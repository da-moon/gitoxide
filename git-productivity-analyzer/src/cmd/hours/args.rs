use crate::cmd::common::CommonArgs;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub common: CommonArgs,

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

crate::impl_from_args!(
    Args,
    crate::sdk::hours::Options {
        no_bots,
        file_stats,
        line_stats,
        show_pii,
        omit_unify_identities,
        threads
    }
);
