use crate::sdk::branch::delete_branch;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    pub name: String,
}

pub fn run(repo: &gix::Repository, args: Args) -> crate::error::Result<()> {
    delete_branch(repo, &args.name)
}
