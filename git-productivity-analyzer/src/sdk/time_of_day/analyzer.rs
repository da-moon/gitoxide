use crate::{error::Result, Globals};
use gix::bstr::ByteSlice;
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

pub struct Analyzer {
    opts: Options,
    globals: Globals,
}

impl Analyzer {
    pub fn new(opts: Options, globals: Globals) -> Self {
        Self { opts, globals }
    }

    pub fn analyze(self) -> Result<Histogram> {
        let repo = gix::discover(&self.opts.working_dir).into_diagnostic()?;
        let start = self.resolve_start_commit(&repo)?;
        let since = self.resolve_since_commit(&repo)?;

        let mut bins = vec![0u32; self.opts.bins as usize];
        self.walk_commits(&repo, start, since.as_ref(), &mut bins)?;

        Ok(Histogram { counts: bins })
    }

    fn resolve_start_commit(&self, repo: &gix::Repository) -> Result<gix::ObjectId> {
        let spec = self.globals.until.as_deref().unwrap_or(&self.opts.rev_spec);
        Ok(repo
            .rev_parse_single(spec.as_bytes().as_bstr())
            .into_diagnostic()?
            .detach())
    }

    fn resolve_since_commit(&self, repo: &gix::Repository) -> Result<Option<gix::ObjectId>> {
        match &self.globals.since {
            Some(spec) => Ok(Some(
                repo.rev_parse_single(spec.as_bytes().as_bstr())
                    .into_diagnostic()?
                    .detach(),
            )),
            None => Ok(None),
        }
    }

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
            let bin_size = 24.0 / hist.counts.len() as f32;
            for (i, count) in hist.counts.iter().enumerate() {
                let start = (i as f32 * bin_size).round() as u32;
                let end = ((i + 1) as f32 * bin_size).round() as u32;
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
