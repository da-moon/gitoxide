use crate::sdk::branch::cleanup_merged_branches;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(long, help = "Only show what would be deleted")]
    pub dry_run: bool,
}

pub fn run(repo: &gix::Repository, args: Args) -> crate::error::Result<()> {
    for name in cleanup_merged_branches(repo, args.dry_run)? {
        if args.dry_run {
            println!("would delete {name}");
        } else {
            println!("deleted {name}");
        }
    }
    Ok(())
}
