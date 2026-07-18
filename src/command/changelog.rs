use crate::cli::Cli;
use crate::error::CliError;
use serde_json::Value;

pub async fn handle(count: usize, cli: &Cli) -> Result<Value, CliError> {
    crate::handler::changelog::handle(count, cli).await
}
