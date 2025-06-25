use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(Debug, Clone, ClapArgs)]
pub struct CommonArgs {
    #[arg(
        long = "working-dir",
        default_value = ".",
        help = "The directory containing a '.git/' folder."
    )]
    pub working_dir: PathBuf,

    #[arg(
        long = "rev-spec",
        default_value = "HEAD",
        help = "The revision to start walking from."
    )]
    pub rev_spec: String,
}

#[macro_export]
macro_rules! impl_from_args {
    ($args:ty, $opts:ty { $($field:ident),* }) => {
        impl From<$args> for $opts {
            fn from(a: $args) -> Self {
                Self {
                    repo: a.common.into(),
                    $($field: a.$field,)*
                }
            }
        }
    };
}
