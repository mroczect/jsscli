use crate::cli::Cli;
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(
    method: String,
    args: Option<Vec<String>>,
    cli: &Cli,
) -> Result<Value, CliError> {
    handler::call_method::handle(cli, &method, args).await
}
