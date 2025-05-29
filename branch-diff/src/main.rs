use anyhow::{Context, Result};
use clap::Parser;
use gix::object::tree::diff::{Change, ChangeDetached};
use gix::prelude::*;

/// Show file changes in a branch relative to a base branch.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the repository
    #[arg(default_value = ".")]
    repo: std::path::PathBuf,

    /// Name of the branch to compare
    branch: String,

    /// Name of the base branch
    #[arg(default_value = "master")]
    base: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let repo = gix::discover(&args.repo).context("open repository")?;

    let lhs_id = repo
        .rev_parse_single(format!("refs/heads/{}", args.base))
        .context("resolve base branch")?;
    let rhs_id = repo
        .rev_parse_single(format!("refs/heads/{}", args.branch))
        .context("resolve branch")?;

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
