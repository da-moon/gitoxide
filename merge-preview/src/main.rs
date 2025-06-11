use std::io::Write;

use anyhow::Result;
use clap::Parser;
use gix::{
    bstr::{BString, ByteSlice},
    merge::{blob::builtin_driver::text::Labels, tree::TreatAsUnresolved},
};

/// Preview merging another branch without affecting the repository.
#[derive(Debug, Parser)]
#[command(name = "merge-preview", about = "preview a merge of another branch", version = option_env!("GIX_VERSION"))]
struct Args {
    /// branch or revision to merge into HEAD. defaults to @{u}
    #[arg(value_name = "target")]
    target: Option<String>,
}

fn main() -> Result<()> {
    run(Args::parse_from(gix::env::args_os()))
}

fn run(args: Args) -> Result<()> {
    let repo = gix::open(".")?;
    let ours: gix_hash::ObjectId = repo.head()?.into_peeled_id()?.into();
    let target_spec = args.target.as_deref().unwrap_or("@{u}");
    let theirs: gix_hash::ObjectId = repo
        .rev_parse_single(target_spec)?
        .object()?
        .peel_to_commit()?
        .id
        .into();

    let current_label: BString = "HEAD".into();
    let other_label: BString = target_spec.into();
    let labels = Labels {
        ancestor: None,
        current: Some(current_label.as_bstr()),
        other: Some(other_label.as_bstr()),
    };

    let outcome = repo.merge_commits(ours, theirs, labels, repo.tree_merge_options()?.into())?;
    if outcome.tree_merge.has_unresolved_conflicts(TreatAsUnresolved::git()) {
        println!("Conflicts:");
        for conflict in &outcome.tree_merge.conflicts {
            let path = conflict.changes_in_resolution().1.location();
            println!("  {}", path);
            if let Some(info) = conflict.content_merge() {
                if info.resolution == gix::merge::blob::Resolution::Conflict {
                    let blob = repo.find_blob(info.merged_blob_id)?;
                    std::io::stdout().write_all(&blob.data)?;
                }
            }
        }
    } else {
        println!("Merge would apply without conflicts.");
    }
    Ok(())
}
