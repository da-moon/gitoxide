use crate::{error::Result, Globals};

use super::processor::process_commit;

/// A histogram of commit counts grouped into time-of-day buckets.
pub struct Histogram {
    counts: Vec<u32>,
}

impl Histogram {
    /// Returns the number of commits per bin.
    pub fn counts(&self) -> &[u32] {
        &self.counts
    }
}

#[derive(Clone)]
pub struct Options {
    pub repo: crate::sdk::RepoOptions,
    pub bins: u8,
    pub author: Option<String>,
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

impl Analyzer {
    pub fn analyze(self) -> Result<Histogram> {
        if self.opts.bins == 0 || self.opts.bins > 24 {
            return Err(miette::miette!("--bins must be in 1..=24"));
        }

        let (repo, start, since) = crate::sdk::open_with_range(&self.opts.repo, &self.globals)?;

        let mut bins = vec![0u32; self.opts.bins as usize];
        self.walk_commits(&repo, start, since.as_ref(), &mut bins)?;

        Ok(Histogram { counts: bins })
    }

    // commit range resolution is provided by sdk::revision

    fn walk_commits(
        &self,
        repo: &gix::Repository,
        start: gix::ObjectId,
        since: Option<&gix::ObjectId>,
        bins: &mut [u32],
    ) -> Result<()> {
        crate::sdk::walk_commits(repo, start, since, |_, commit| {
            process_commit(commit, &self.opts.author, bins)
        })
    }

    pub fn print_histogram(&self, hist: &Histogram) {
        crate::sdk::print_json_or(self.globals.json, &SerializableHistogram::from(hist), || {
            let bins = hist.counts().len() as u32;
            for (i, count) in hist.counts().iter().enumerate() {
                // Compute bin boundaries so that bin assignment matches the processor.
                // The processor places hour `h` into `h * bins / 24`, so here we invert
                // that mapping using integer arithmetic only.
                let start = (i as u32 * 24).div_ceil(bins);
                let mut end = ((i as u32 + 1) * 24).div_ceil(bins) - 1;
                if i as u32 == bins - 1 {
                    end = 23;
                }
                println!("{:02}-{:02}: {count}", start, end);
            }
        });
    }
}

#[derive(serde::Serialize)]
struct SerializableHistogram {
    bins: Vec<u32>,
}

impl From<&Histogram> for SerializableHistogram {
    fn from(h: &Histogram) -> Self {
        Self { bins: h.counts.clone() }
    }
}

crate::impl_analyzer_boilerplate!(Options, Analyzer);

impl crate::sdk::AnalyzerTrait for Analyzer {
    type Output = Histogram;
    fn analyze(self) -> crate::error::Result<Self::Output> {
        Analyzer::analyze(self)
    }
}
