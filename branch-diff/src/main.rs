use anyhow::{Context, Result};
use clap::Parser;
use gix::object::tree::diff::{Change, ChangeDetached};
use gix::prelude::*;
use gix::Repository;
use gix_hash::ObjectId;

/// Show file changes in a branch relative to a base branch.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the repository
    #[arg(default_value = ".")]
    repo: std::path::PathBuf,

    /// Name of the branch to compare. If omitted, the current branch is used.
    branch: Option<String>,

    /// Name of the base branch. If omitted, it will be detected automatically.
    base: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let repo = gix::discover(&args.repo).context("open repository")?;

    let branch = match args.branch {
        Some(b) => b,
        None => repo
            .head_name()?
            .context("current branch is detached")?
            .to_string()
            .trim_start_matches("refs/heads/")
            .to_string(),
    };

    let rhs_id = repo
        .rev_parse_single(format!("refs/heads/{}", branch))
        .context("resolve branch")?;

    let base = match args.base {
        Some(b) => Some(b),
        None => detect_base_branch(&repo, &branch, rhs_id.detach())?,
    };

    let base = base.unwrap_or_else(|| "master".to_string());

    let lhs_id = repo
        .rev_parse_single(format!("refs/heads/{}", base))
        .context("resolve base branch")?;

    let lhs_commit = repo.find_commit(lhs_id)?;
    let rhs_commit = repo.find_commit(rhs_id)?;

    let lhs_tree = lhs_commit.tree()?;
    let rhs_tree = rhs_commit.tree()?;

    let changes: Vec<ChangeDetached> =
        repo.diff_tree_to_tree(Some(&lhs_tree), Some(&rhs_tree), None)?;

    for change in changes {
        let change = change.attach(&repo, &repo);
        match change {
            Change::Addition { location, .. } => {
                println!("A\t{}", location.to_string_lossy());
            }
            Change::Deletion { location, .. } => {
                println!("D\t{}", location.to_string_lossy());
            }
            Change::Modification { location, .. } => {
                println!("M\t{}", location.to_string_lossy());
            }
            Change::Rewrite { location, source_location, .. } => {
                println!(
                    "R\t{} -> {}",
                    source_location.to_string_lossy(),
                    location.to_string_lossy()
                );
            }
        }
    }
    Ok(())
}

/// Try to detect the branch from which `branch` diverged.
fn detect_base_branch(repo: &Repository, branch: &str, head_id: gix_hash::ObjectId) -> Result<Option<String>> {
    let mut refs = repo.references()?.local_branches()?.peeled()?;
    let mut best: Option<(String, i64)> = None;
    for reference in refs {
        let mut reference = reference?;
        let name = reference.name().to_string();
        if name.trim_start_matches("refs/heads/") == branch {
            continue;
        }
        let target = reference.peel_to_id_in_place()?.detach();
        if let Ok(base) = repo.merge_base(head_id, target) {
            if base == target {
                let commit = repo.find_commit(target)?;
                let time = commit.time()?.seconds;
                if best.as_ref().map_or(true, |(_, t)| time > *t) {
                    best = Some((name, time));
                }
            }
        }
    }
    Ok(best.map(|(name, _)| name.trim_start_matches("refs/heads/").to_string()))
}
