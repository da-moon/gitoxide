pub mod cleanup;
pub mod compare;
pub mod create;
pub mod delete;
pub mod list;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "List branches")]
    List(list::Args),
    #[command(about = "Create a new branch")]
    Create(create::Args),
    #[command(about = "Delete a branch")]
    Delete(delete::Args),
    #[command(about = "Compare branches")]
    Compare(compare::Args),
    #[command(about = "Cleanup merged branches")]
    Cleanup(cleanup::Args),
}
