use crate::sdk::diff::{commit_trees, configure_changes, create_changes};
use crate::sdk::stats::{median, percentile_of_sorted};
use crate::{error::Result, Globals};
use miette::IntoDiagnostic;
use serde::Serialize;

#[derive(Serialize)]
pub struct Summary {
    pub min_files: u32,
    pub max_files: u32,
    pub avg_files: f64,
    pub median_files: f64,
    pub min_lines: u32,
    pub max_lines: u32,
    pub avg_lines: f64,
    pub median_lines: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_percentiles: Option<Vec<(f64, u32)>>,
}

#[derive(Clone)]
pub struct Options {
    pub repo: crate::sdk::RepoOptions,
    pub percentiles: Option<Vec<f64>>,
}

impl Options {
    pub fn validate(&self) -> crate::error::Result<()> {
        if let Some(pcts) = &self.percentiles {
            if let Some(p) = pcts.iter().copied().find(|p| !(0.0..=100.0).contains(p)) {
                return Err(miette::miette!("percentile {p} out of range 0..=100"));
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl Analyzer {
    pub fn analyze(self) -> Result<Summary> {
        let (repo, start, since) = crate::sdk::open_with_range(&self.opts.repo, &self.globals)?;
        let mut lines = Vec::new();
        let mut files = Vec::new();
        crate::sdk::walk_commits(&repo, start, since.as_ref(), false, |id, commit| {
            self.process_commit(&repo, id, &commit, &mut lines, &mut files)
        })?;
        Ok(self.build_summary(lines, files))
    }

    fn process_commit(
        &self,
        repo: &gix::Repository,
        commit_id: gix::ObjectId,
        commit: &gix::Commit<'_>,
        lines: &mut Vec<u32>,
        files: &mut Vec<u32>,
    ) -> Result<()> {
        let parent = commit.parent_ids().next().map(|id| id.detach());
        let (from, to) = commit_trees(repo, commit_id, parent);
        let mut diff = create_changes(&from)?;
        configure_changes(&mut diff);
        let stats = diff.stats(&to).into_diagnostic()?;
        files.push(stats.files_changed as u32);
        lines.push((stats.lines_added + stats.lines_removed) as u32);
        Ok(())
    }

    fn build_summary(&self, mut lines: Vec<u32>, files: Vec<u32>) -> Summary {
        let min_files = files.iter().copied().min().unwrap_or(0);
        let max_files = files.iter().copied().max().unwrap_or(0);
        let avg_files = if files.is_empty() {
            0.0
        } else {
            files.iter().copied().map(f64::from).sum::<f64>() / files.len() as f64
        };
        let mut files_sorted = files.clone();
        files_sorted.sort_unstable();
        let median_files = median(&files_sorted);
        let min_lines = lines.iter().copied().min().unwrap_or(0);
        let max_lines = lines.iter().copied().max().unwrap_or(0);
        let avg_lines = if lines.is_empty() {
            0.0
        } else {
            lines.iter().copied().map(f64::from).sum::<f64>() / lines.len() as f64
        };
        let mut lines_sorted = lines.clone();
        lines_sorted.sort_unstable();
        let median_lines = median(&lines_sorted);
        let line_percentiles = if lines.is_empty() {
            None
        } else {
            self.opts.percentiles.as_ref().map(|pcts| {
                lines.sort_unstable();
                pcts.iter()
                    .map(|p| (*p, percentile_of_sorted(&lines, *p).unwrap_or_default()))
                    .collect()
            })
        };
        Summary {
            min_files,
            max_files,
            avg_files,
            median_files,
            min_lines,
            max_lines,
            avg_lines,
            median_lines,
            line_percentiles,
        }
    }

    pub fn print_summary(&self, summary: &Summary) {
        crate::sdk::print_json_or(self.globals.json, summary, || {
            println!(
                "files per commit: min={} max={} avg={:.2} median={:.2}",
                summary.min_files, summary.max_files, summary.avg_files, summary.median_files
            );
            println!(
                "lines per commit: min={} max={} avg={:.2} median={:.2}",
                summary.min_lines, summary.max_lines, summary.avg_lines, summary.median_lines
            );
            if let Some(pcts) = &summary.line_percentiles {
                for (p, value) in pcts {
                    println!("p{} = {}", p, value);
                }
            }
        });
    }
}

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Summary;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
