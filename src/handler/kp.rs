use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::Value;
use std::path::PathBuf;

pub async fn handle(cli: &Cli, no_perjanjian: &str) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let raw = client
        .doctype("Master Data Nasabah")
        .filter("no_perjanjian", "=", no_perjanjian)
        .fields(vec!["name"])
        .limit(1)
        .execute_raw()
        .await?;

    let data: Value = serde_json::from_str(&raw)?;
    let docname = data["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|doc| doc["name"].as_str())
        .ok_or_else(|| {
            CliError::Other(format!(
                "No nasabah found with no_perjanjian: {}",
                no_perjanjian
            ))
        })?;

    let pdf_bytes = client
        .download_pdf_kartu_piutang(
            "Master Data Nasabah",
            docname,
            "Form Rincian Sisa Piutang Nasabah",
            true,
        )
        .await?;

    let download_dir = dirs_next::download_dir().unwrap_or_else(|| PathBuf::from("."));
    let filename = format!("Rincian_Sisa_Piutang_{}.pdf", no_perjanjian);
    let save_path = download_dir.join(filename);

    std::fs::write(&save_path, pdf_bytes).map_err(|e| CliError::Io(e))?;

    Ok(serde_json::json!({
        "ok": true,
        "path": save_path.display().to_string(),
        "message": format!("PDF downloaded to {}", save_path.display())
    }))
}
