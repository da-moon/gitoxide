use crate::{error::Result, Globals};
use gix::prelude::*;
use miette::IntoDiagnostic;
use std::path::PathBuf;

use super::processor::process_commit;

pub struct Histogram {
    pub counts: Vec<u32>,
}

#[derive(Clone)]
pub struct Options {
    pub working_dir: PathBuf,
    pub rev_spec: String,
    pub bins: u8,
    pub author: Option<String>,
}

impl Options {
    pub fn into_analyzer(self, globals: Globals) -> Analyzer {
        Analyzer::new(self, globals)
    }
}

#[derive(Clone)]
pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

impl Analyzer {
    pub fn new(opts: Options, globals: Globals) -> Self {
        Self { opts, globals }
    }

    pub fn analyze(self) -> Result<Histogram> {
        if self.opts.bins == 0 {
            return Err(miette::miette!("--bins must be greater than 0"));
        }

        let repo = gix::discover(&self.opts.working_dir).into_diagnostic()?;
        let start = crate::sdk::resolve_start_commit(&repo, &self.opts.rev_spec, self.globals.until.as_deref())?;
        let since = crate::sdk::resolve_since_commit(&repo, self.globals.since.as_deref())?;

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
        let iter = start.ancestors(&repo.objects);
        let mut buf = Vec::new();
        for item in iter {
            let info = item.into_diagnostic()?;
            let commit = repo.objects.find_commit_iter(&info.id, &mut buf).into_diagnostic()?;
            process_commit(commit, &self.opts.author, bins)?;
            if let Some(id) = since {
                if &info.id == id {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn print_histogram(&self, hist: &Histogram) {
        if self.globals.json {
            let ser = SerializableHistogram::from(hist);
            let _ = serde_json::to_writer(std::io::stdout(), &ser).map(|_| println!());
        } else {
            let bins = hist.counts.len() as u32;
            for (i, count) in hist.counts.iter().enumerate() {
                let start = i as u32 * 24 / bins;
                let end = (i as u32 + 1) * 24 / bins;
                println!("{:02}-{:02}: {count}", start, end - 1);
            }
        }
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
