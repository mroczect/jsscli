use crate::cli::Cli;
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(number: String, cli: &Cli) -> Result<Value, CliError> {
    handler::files::handle(cli, &number).await
}
