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

/// Implements `From<$args>` for `$opts`.
///
/// Each field may optionally specify:
/// - A source field using `dest => src` (defaults to same name if omitted)
/// - A transformation using `| transform` (currently supports `lowercase`)
///
/// ```
/// impl_from_args!(Args, Options { foo, bar => other });
/// // expands to `foo: a.foo` and `bar: a.other`.
/// 
/// impl_from_args!(Args, Options { foo, bar => other | lowercase });
/// // expands to `foo: a.foo` and `bar: a.other.map(|s| s.to_lowercase())`.
/// ```
#[macro_export]
macro_rules! impl_from_args {
    // Helper: no-op if no transform
    (@apply $expr:expr, ) => { $expr };
    // Helper: lowercase transform
    (@apply $expr:expr, lowercase) => { $expr.map(|s| s.to_lowercase()) };
    
    // Helper: get source field (defaults to dest if not specified)
    (@source $a:ident, $dest:ident, $src:ident) => { $a.$src };
    (@source $a:ident, $dest:ident, ) => { $a.$dest };

    // Single outer arm: each field may have `=> src` and/or `| transform`
    ($Args:ty, $Opts:ty {
        $(
            $dest:ident
            $( => $src:ident )?
            $( | $trans:ident )?
        ),* $(,)?
    }) => {
        impl From<$Args> for $Opts {
            fn from(a: $Args) -> Self {
                Self {
                    repo: a.common.into(),
                    $(
                        $dest: $crate::impl_from_args!{
                            @apply
                            $crate::impl_from_args!(@source a, $dest, $( $src )?),
                            $( $trans )?
                        },
                    )*
                }
            }
        }
    };
}
