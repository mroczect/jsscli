pub mod account;
pub mod auth;
pub mod config;
pub mod doctype;
pub mod file;

use crate::cli::{Cli, Command};
use crate::error::CliResult;

pub async fn dispatch(cli: Cli) -> CliResult<()> {
    match &cli.command {
        Command::Auth { cmd } => auth::handle(cmd.clone(), &cli).await,
        Command::Account { cmd } => account::handle(cmd.clone(), &cli).await,
        Command::Config { cmd } => config::handle(cmd.clone(), &cli).await,
        Command::Doctype { cmd } => doctype::handle(cmd.clone(), &cli).await,
        Command::File { cmd } => file::handle(cmd.clone(), &cli).await,
    }
}
