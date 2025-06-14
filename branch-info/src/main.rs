use anyhow::{Context, Result};
use clap::Parser;
use gix::bstr::ByteSlice;
use std::path::PathBuf;

use gix::object::tree::diff::{Change, ChangeDetached};
use gix::prelude::TreeDiffChangeExt;
use std::borrow::ToOwned;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = ".", help = "Path to the repository")]
    repo: PathBuf,

    #[arg(long, help = "Optional starting commit for the diff or range")]
    from: Option<String>,

    #[arg(long, help = "Optional end commit for the diff or range")]
    to: Option<String>,

    #[arg(long, help = "Show file diffs instead of just listing commits")]
    show_diff: bool,
}

fn main() -> Result<()> {
    Args::parse().run()
}

impl Args {
    fn run(self) -> Result<()> {
        let repo = gix::discover(&self.repo).context("open repository")?;

        let head_id = repo.head_id()?;
        let to = self
            .to
            .as_deref()
            .map(|rev| repo.rev_parse_single(rev))
            .transpose()?
            .unwrap_or(head_id);

        let from = self
            .from
            .as_deref()
            .map(|rev| repo.rev_parse_single(rev))
            .transpose()?
            .unwrap_or(detect_base(&repo, head_id)?);

        repo.head_name()? // Option<FullName>
            .map(|name| {
                name.shorten()
                    .to_str()
                    .context("branch name invalid UTF-8")
                    .map(ToOwned::to_owned)
            })
            .transpose()? // Result<Option<String>>
            .map_or_else(|| println!("HEAD is detached"), |branch| println!("branch: {branch}"));

        if self.show_diff {
            println!("diff between {} and {}:", from.shorten_or_id(), to.shorten_or_id());
            show_patch(&repo, from, to)?;
        } else {
            list_commits(&repo, from, to)?;
        }
        Ok(())
    }
}

fn detect_base<'repo>(repo: &'repo gix::Repository, head: gix::Id<'repo>) -> Result<gix::Id<'repo>> {
    repo.head_name()? // Option<FullName>
        .and_then(|name| repo.branch_remote_ref_name(name.as_ref(), gix::remote::Direction::Fetch))
        .transpose()? // Result<Option<Cow<FullNameRef>>> -> Option<Result<...>>
        .map(|up| repo.find_reference(up.as_ref()))
        .transpose()? // Result<Option<Reference>>
        .map(|mut r| r.peel_to_id_in_place())
        .transpose()? // Result<Option<Id>>
        .map(|upstream_id| repo.merge_base(head, upstream_id))
        .transpose()? // Result<Option<Id>>
        .map_or(Ok(head), Ok)
}

fn list_commits(repo: &gix::Repository, from: gix::Id<'_>, to: gix::Id<'_>) -> Result<()> {
    let commits = repo.rev_walk([to]).with_boundary([from]).all()?;
    for info in commits {
        let info = info?;
        let commit = info.object()?;
        let summary = commit.message()?.summary();
        println!("{} {}", info.id, summary.to_str_lossy());
    }
    Ok(())
}

fn show_patch(repo: &gix::Repository, lhs: gix::Id<'_>, rhs: gix::Id<'_>) -> Result<()> {
    let lhs_commit = repo.find_commit(lhs)?;
    let rhs_commit = repo.find_commit(rhs)?;

    let lhs_tree = lhs_commit.tree()?;
    let rhs_tree = rhs_commit.tree()?;

    let changes: Vec<ChangeDetached> = repo.diff_tree_to_tree(Some(&lhs_tree), Some(&rhs_tree), None)?;

    let mut cache = repo.diff_resource_cache_for_tree_diff()?;

    for change in changes {
        let change = change.attach(repo, repo);
        print_patch(&mut cache, change)?;
    }
    Ok(())
}

fn print_patch(cache: &mut gix::diff::blob::Platform, change: Change<'_, '_, '_>) -> Result<()> {
    use gix::diff::blob::platform::prepare_diff::Operation;
    use gix::diff::blob::{
        unified_diff::{ContextSize, NewlineSeparator},
        UnifiedDiff,
    };
    let (old_path, new_path) = match &change {
        Change::Rewrite {
            source_location,
            location,
            ..
        } => (source_location.as_bstr(), location.as_bstr()),
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
