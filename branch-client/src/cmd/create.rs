use crate::sdk::branch::create_branch;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    pub name: String,
    #[arg(long, default_value = "HEAD", help = "Starting point for the branch")]
    pub start: String,
}

pub fn run(repo: &gix::Repository, args: Args) -> crate::error::Result<()> {
    create_branch(repo, &args.name, &args.start)
}
