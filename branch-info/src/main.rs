use clap::Parser;
use gix::{bstr::ByteSlice, remote};
use std::process::Command;

/// Display information about the current branch and its upstream.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the repository
    #[arg(long, default_value = ".")]
    repo: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Args::parse())
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let repo = gix::discover(&args.repo)?;
    if let Some(head) = repo.head_name()? {
        println!(
            "Current branch: {}",
            head.shorten().to_str_lossy()
        );
        match repo.branch_remote_tracking_ref_name(head.as_ref(), remote::Direction::Fetch) {
            Some(Ok(name)) => {
                println!("Upstream branch: {}", name.shorten().to_str_lossy());

                let upstream_commit = repo
                    .find_reference(name.as_ref())?
                    .peel_to_commit()?;
                let head_commit = repo.head_commit()?;
                let base_id = repo.merge_base(head_commit.id, upstream_commit.id)?.id;

                println!(
                    "Commits since branching point starting from {}..HEAD:",
                    base_id
                );
                let output = Command::new("git")
                    .current_dir(repo.workdir().unwrap_or_else(|| repo.git_dir()))
                    .arg("rev-list")
                    .arg(format!("{}..HEAD", base_id))
                    .output()?;
                if output.status.success() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                }
            }
            Some(Err(err)) => println!("Failed to get upstream: {err}"),
            None => println!("No upstream configured"),
        }
    } else {
        println!("HEAD is detached");
    }
    Ok(())
}
