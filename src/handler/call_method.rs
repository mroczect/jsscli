use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::Value;
use std::collections::HashMap;

pub async fn handle(cli: &Cli, method: &str, args: Option<Vec<String>>) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let mut params = HashMap::new();
    if let Some(args) = args {
        for arg in args {
            if let Some((k, v)) = arg.split_once('=') {
                params.insert(k.to_string(), v.to_string());
            } else {
                return Err(CliError::Usage(format!(
                    "Invalid argument `{arg}`. Use key=value format."
                )));
            }
        }
    }

    let resp = client.call_method(method, Some(params)).await?;
    let v: Value = serde_json::from_str(&resp)?;
    Ok(v)
}
