use crate::cli::{AuthCommand, Cli};
use crate::error::CliResult;
use crate::handler;

pub async fn handle(cmd: AuthCommand, cli: &Cli) -> CliResult<()> {
    match cmd {
        AuthCommand::Login => handler::auth::login(cli).await,
        AuthCommand::Logout => handler::auth::logout(cli).await,
        AuthCommand::Status => handler::auth::status(cli).await,
        AuthCommand::Whoami => handler::auth::whoami(cli).await,
    }
}
