use crate::cli::Cli;
use crate::error::CliError;
use serde_json::Value;

pub async fn handle(cli: &Cli) -> Result<Value, CliError> {
    crate::handler::info::handle(cli).await
}
