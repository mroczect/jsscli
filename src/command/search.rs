use crate::cli::Cli;
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(
    query: String,
    limit: u32,
    doctype: Option<String>,
    cli: &Cli,
) -> Result<Value, CliError> {
    handler::search::handle(cli, &query, limit, doctype.as_deref()).await
}
