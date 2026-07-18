use crate::cli::Cli;
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(cli: &Cli, no_perjanjian: &str) -> Result<Value, CliError> {
    handler::kp::handle(cli, no_perjanjian).await
}
