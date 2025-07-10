use crate::{error::Result, util::spawn_blocking, Globals};
use gix::bstr::ByteSlice;
use serde::Serialize;

/// Print either JSON or human-readable output based on the [`json`] flag.
pub fn print_json_or<T>(json: bool, value: &T, human: impl FnOnce())
where
    T: Serialize,
{
    if json {
        let _ = serde_json::to_writer(std::io::stdout(), value).map(|_| println!());
    } else {
        human();
    }
}

/// Trait implemented by analyzers which produce an output.
pub trait AnalyzerTrait: Clone + Send + 'static {
    type Output: Send + 'static;
    fn analyze(self) -> Result<Self::Output>;
}

/// Trait for option types that can create analyzers.
pub trait IntoAnalyzer {
    type Analyzer: AnalyzerTrait;
    fn into_analyzer(self, globals: Globals) -> Self::Analyzer;
}

/// Execute [`AnalyzerTrait::analyze`] in a blocking task and print the result.
pub async fn run_with_analyzer<O>(
    opts: O,
    globals: &Globals,
    print: impl FnOnce(&O::Analyzer, &<O::Analyzer as AnalyzerTrait>::Output) + Send + 'static,
) -> Result<()>
where
    O: IntoAnalyzer + Send + 'static,
{
    let analyzer = opts.into_analyzer(globals.clone());
    let worker = analyzer.clone();
    let output = spawn_blocking(move || worker.analyze()).await?;
    print(&analyzer, &output);
    Ok(())
}

#[macro_export]
macro_rules! impl_analyzer_boilerplate {
    ($opts:path, $analyzer:path) => {
        impl $crate::sdk::IntoAnalyzer for $opts {
            type Analyzer = $analyzer;
            fn into_analyzer(self, globals: $crate::Globals) -> Self::Analyzer {
                <$analyzer>::new(self, globals)
            }
        }
        impl $analyzer {
            pub fn new(opts: $opts, globals: $crate::Globals) -> Self {
                Self { opts, globals }
            }
        }
    };
}

/// Return `true` if `author` matches the optional `filter` string.
pub fn author_matches(author: &gix::actor::SignatureRef<'_>, filter: &Option<String>) -> bool {
    match filter {
        Some(pattern) => {
            let pat = pattern.to_lowercase();
            let name = author.name.to_str_lossy().to_lowercase();
            let email = author.email.to_str_lossy().to_lowercase();
            name.contains(&pat) || email.contains(&pat)
        }
        None => true,
    }
}
