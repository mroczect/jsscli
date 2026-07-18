pub mod account;
pub mod auth;
pub mod changelog;
pub mod config;
pub mod doctype;
pub mod file;
pub mod info;
pub mod manual;

use crate::cli::{Cli, Command};
use crate::error::CliError;
use serde_json::Value;

pub async fn dispatch(cli: Cli) -> Result<Value, CliError> {
    match &cli.command {
        Command::Auth { cmd } => auth::handle(cmd.clone(), &cli).await,
        Command::Account { cmd } => account::handle(cmd.clone(), &cli).await,
        Command::Config { cmd } => config::handle(cmd.clone(), &cli).await,
        Command::Doctype { cmd } => doctype::handle(cmd.clone(), &cli).await,
        Command::File { cmd } => file::handle(cmd.clone(), &cli).await,
        Command::Info => info::handle(&cli).await,
        Command::Changelog { count } => changelog::handle(*count, &cli).await,
        Command::Manual => manual::handle(&cli).await,
    }
}
