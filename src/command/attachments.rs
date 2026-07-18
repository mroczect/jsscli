use crate::cli::Cli;
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(doctype: String, name: String, cli: &Cli) -> Result<Value, CliError> {
    handler::attachments::handle(cli, &doctype, &name).await
}
