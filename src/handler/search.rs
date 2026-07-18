use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::Value;

pub async fn handle(
    cli: &Cli,
    query: &str,
    limit: u32,
    doctype: Option<&str>,
) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let body = client.global_search(query, limit, doctype).await?;
    let v: Value = serde_json::from_str(&body)?;
    Ok(v)
}
