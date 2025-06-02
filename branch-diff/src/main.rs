use anyhow::{anyhow, Context, Result};
use clap::Parser;
use gix::bstr::ByteSlice;
use gix::diff::blob::{unified_diff::{ContextSize, NewlineSeparator}, UnifiedDiff};
use gix::object::tree::diff::{Change, ChangeDetached};
use gix::prelude::*;
use gix::hash::ObjectId;
use gix::Repository;

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

    /// Output unified diff instead of a summary.
    #[arg(long)]
    patch: bool,
}

fn main() -> Result<()> {
    run(Args::parse())
}

fn run(args: Args) -> Result<()> {
    let repo = gix::discover(&args.repo).context("open repository")?;
    let branch = current_branch(&repo, args.branch)?;
    let rhs_id = resolve_ref(&repo, &branch).context("resolve branch")?;

    let base = detect_or_default_base(&repo, args.base, &branch, rhs_id)?;
    let lhs_id = resolve_ref(&repo, &base).context("resolve base branch")?;

    if args.patch {
        show_patch(&repo, lhs_id, rhs_id)
    } else {
        show_summary(&repo, lhs_id, rhs_id)
    }
}

fn current_branch(repo: &Repository, branch_arg: Option<String>) -> Result<String> {
    Ok(match branch_arg {
        Some(b) => b,
        None => repo
            .head_name()?
            .context("current branch is detached")?
            .shorten()
            .to_str()
            .context("branch name invalid UTF-8")?
            .to_owned(),
    })
}

fn resolve_ref(repo: &Repository, name: &str) -> Result<ObjectId> {
    let spec = format!("refs/heads/{}", name);
    Ok(repo.rev_parse_single(spec.as_str())?.detach())
}

fn detect_or_default_base(
    repo: &Repository,
    base_arg: Option<String>,
    branch: &str,
    head_id: ObjectId,
) -> Result<String> {
    match base_arg {
        Some(b) => Ok(b),
        None => detect_base_branch(repo, branch, head_id)?
            .ok_or_else(|| anyhow!("no base branch found"))
            .or_else(|_| Ok("master".to_string())),
    }
}

fn show_summary(repo: &Repository, lhs_id: ObjectId, rhs_id: ObjectId) -> Result<()> {
    let lhs_commit = repo.find_commit(lhs_id)?;
    let rhs_commit = repo.find_commit(rhs_id)?;

    let lhs_tree = lhs_commit.tree()?;
    let rhs_tree = rhs_commit.tree()?;

    let changes: Vec<ChangeDetached> =
        repo.diff_tree_to_tree(Some(&lhs_tree), Some(&rhs_tree), None)?;

    for change in changes {
        let change = change.attach(repo, repo);
        match change {
            Change::Addition { location, .. } => {
                println!("A\t{}", location.to_str_lossy());
            }
            Change::Deletion { location, .. } => {
                println!("D\t{}", location.to_str_lossy());
            }
            Change::Modification { location, .. } => {
                println!("M\t{}", location.to_str_lossy());
            }
            Change::Rewrite { location, source_location, .. } => {
                println!(
                    "R\t{} -> {}",
                    source_location.to_str_lossy(),
                    location.to_str_lossy()
                );
            }
        }
    }
    Ok(())
}


fn show_patch(repo: &Repository, lhs_id: ObjectId, rhs_id: ObjectId) -> Result<()> {
    let lhs_commit = repo.find_commit(lhs_id)?;
    let rhs_commit = repo.find_commit(rhs_id)?;

    let lhs_tree = lhs_commit.tree()?;
    let rhs_tree = rhs_commit.tree()?;

    let changes: Vec<ChangeDetached> =
        repo.diff_tree_to_tree(Some(&lhs_tree), Some(&rhs_tree), None)?;

    let mut cache = repo.diff_resource_cache_for_tree_diff()?;

    for change in changes {
        let change = change.attach(repo, repo);
        print_patch(&mut cache, change)?;
    }
    Ok(())
}

fn print_patch(
    cache: &mut gix::diff::blob::Platform,
    change: Change<'_, '_ , '_>,
) -> Result<()> {
    use gix::diff::blob::platform::prepare_diff::Operation;
    let (old_path, new_path) = match &change {
        Change::Rewrite { source_location, location, .. } => (source_location.as_bstr(), location.as_bstr()),
        Change::Addition { location, .. }
        | Change::Deletion { location, .. }
        | Change::Modification { location, .. } => (location.as_bstr(), location.as_bstr()),
    };
    if change.entry_mode().is_tree() {
        return Ok(());
    }
    println!("diff --git a/{} b/{}", old_path.to_str_lossy(), new_path.to_str_lossy());

    let platform = change.diff(cache)?;
    let outcome = platform.resource_cache.prepare_diff()?;
    let algorithm = match outcome.operation {
        Operation::InternalDiff { algorithm } => algorithm,
        Operation::ExternalCommand { .. } => unreachable!("external diffs disabled"),
        Operation::SourceOrDestinationIsBinary => {
            println!("Binary files differ");
            return Ok(());
        }
    };
    let input = gix::diff::blob::intern::InternedInput::new(
        tokens_for_diffing(outcome.old.data.as_slice().unwrap_or_default()),
        tokens_for_diffing(outcome.new.data.as_slice().unwrap_or_default()),
    );

    let diff = gix::diff::blob::diff(
        algorithm,
        &input,
        UnifiedDiff::new(
            &input,
            Vec::new(),
            NewlineSeparator::AfterHeaderAndLine("\n"),
            ContextSize::symmetrical(3),
        ),
    )?;
    print!("{}", std::str::from_utf8(&diff)?);
    Ok(())
}

fn tokens_for_diffing(data: &[u8]) -> impl gix::diff::blob::intern::TokenSource<Token = &[u8]> {
    gix::diff::blob::sources::byte_lines(data)
}

/// Try to detect the branch from which `branch` diverged.
fn detect_base_branch(repo: &Repository, branch: &str, head_id: ObjectId) -> Result<Option<String>> {
    let refs = repo.references()?;
    let refs = refs.local_branches()?;
    let refs = refs.peeled()?;
    let mut best: Option<(String, i64)> = None;
    for reference in refs {
        let mut reference = reference.map_err(|e| anyhow!(e))?;
        let name = reference
            .name()
            .to_owned()
            .as_bstr()
            .to_str()
            .map_err(|e| anyhow!(e))?
            .to_owned();
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
