use crate::error::Result;
use gitoxide_core::hours::{estimate, Context};
use gix::bstr::ByteSlice;
use miette::IntoDiagnostic;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Summary {
    pub(crate) total_hours: f32,
    pub(crate) total_8h_days: f32,
    pub(crate) total_commits: u32,
    pub(crate) total_authors: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_files: Option<[u32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_lines: Option<[u32; 3]>,
}

#[derive(Clone)]
pub struct Options {
    pub repo: crate::sdk::RepoOptions,
    pub no_bots: bool,
    pub file_stats: bool,
    pub line_stats: bool,
    pub show_pii: bool,
    pub omit_unify_identities: bool,
    pub threads: Option<usize>,
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: crate::Globals,
}

impl Analyzer {
    pub fn analyze(self) -> Result<(Summary, String)> {
        let mut out_buf = Vec::new();
        let spec = self.globals.until.as_deref().unwrap_or(&self.opts.repo.rev_spec);
        estimate(
            &self.opts.repo.working_dir,
            spec.as_bytes().as_bstr(),
            &mut gix::progress::Discard,
            Context {
                show_pii: self.opts.show_pii,
                ignore_bots: self.opts.no_bots,
                file_stats: self.opts.file_stats,
                line_stats: self.opts.line_stats,
                omit_unify_identities: self.opts.omit_unify_identities,
                threads: self.opts.threads,
                out: &mut out_buf,
            },
        )
        .map_err(|e| miette::Report::msg(e.to_string()))?;

        let out_str = std::str::from_utf8(&out_buf).into_diagnostic()?;
        let summary = super::parser::parse_summary(out_str);
        Ok((summary, out_str.to_owned()))
    }

    pub fn print_summary(&self, summary: &Summary, text: &str) {
        crate::sdk::print_json_or(self.globals.json, summary, || {
            // Print the raw output from `estimate` which may include PII
            print!("{}", text);
        });
    }
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = (Summary, String);
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
