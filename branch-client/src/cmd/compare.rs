use crate::sdk::branch::compare_branches;
use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    pub lhs: String,
    pub rhs: String,
}

pub fn run(repo: &gix::Repository, args: Args) -> crate::error::Result<()> {
    let (ahead, behind) = compare_branches(repo, &args.lhs, &args.rhs)?;
    println!("ahead {ahead}, behind {behind}");
    Ok(())
}
