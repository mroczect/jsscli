use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::Value;
use std::collections::HashMap;

pub async fn handle(cli: &Cli, no_rekening: &str) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let mut params = HashMap::new();
    params.insert("no_rekening_kredit".to_string(), no_rekening.to_string());

    let resp = client
        .call_method(
            "juragan.ops.doctype.master_data_nasabah.master_data_nasabah.cek_rincian_hutang_nasabah",
            Some(params),
        )
        .await?;

    let data: Value = serde_json::from_str(&resp)
        .map_err(|e| CliError::Parse(format!("Invalid JSON response: {e}")))?;

    Ok(data)
}
