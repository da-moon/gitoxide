use crate::sdk::branch::list_branches;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[arg(long, help = "List remote branches instead of local")]
    pub remote: bool,
}

pub fn run(repo: &gix::Repository, args: Args) -> crate::error::Result<()> {
    for name in list_branches(repo, args.remote)? {
        println!("{}", name);
    }
    Ok(())
}
