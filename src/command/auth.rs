use crate::cli::{AuthCommand, Cli};
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(cmd: AuthCommand, cli: &Cli) -> Result<Value, CliError> {
    match cmd {
        AuthCommand::Login => handler::auth::login(cli).await,
        AuthCommand::Logout => handler::auth::logout(cli).await,
        AuthCommand::Status => handler::auth::status(cli).await,
        AuthCommand::Whoami => handler::auth::whoami(cli).await,
    }
}
