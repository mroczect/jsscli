use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::{Value, json};
use std::io::{Read, Write};

fn read_json_input(file: &str) -> Result<Value, CliError> {
    let raw = if file == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        let expanded = shellexpand::tilde(file).to_string();
        std::fs::read_to_string(&expanded)?
    };
    serde_json::from_str(&raw)
        .map_err(|e| CliError::Usage(format!("Invalid JSON in `{file}`: {e}")))
}

pub async fn get(cli: &Cli, doctype: &str, name: &str) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let body = client.get_doc(doctype, name).await?;
    let v: Value = serde_json::from_str(&body)?;
    Ok(v)
}

pub async fn list(
    cli: &Cli,
    doctype: &str,
    fields: Option<Vec<String>>,
    filters: Vec<String>,
    limit: u32,
    start: u32,
    order_by: Option<String>,
) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let mut builder = client.doctype(doctype).limit(limit).limit_start(start);
    if let Some(fields) = fields {
        builder = builder.fields(fields.iter().map(|s| s.as_str()).collect());
    }
    if let Some(order) = order_by {
        builder = builder.order_by(&order);
    }
    for f in &filters {
        let parts: Vec<&str> = f.splitn(3, ',').collect();
        if parts.len() != 3 {
            return Err(CliError::Usage(format!(
                "Invalid --filter `{f}`. Expected FIELD,OP,VALUE"
            )));
        }
        builder = builder.filter(parts[0], parts[1], parts[2]);
    }
    let raw = builder.execute_raw().await?;
    serde_json::from_str(&raw).map_err(Into::into)
}

pub async fn create(cli: &Cli, doctype: &str, file: &str) -> Result<Value, CliError> {
    let value = read_json_input(file)?;
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let resp = client.create_doc(doctype, &value).await?;
    let v: Value = serde_json::from_str(&resp)?;
    Ok(json!({"ok": true, "data": v}))
}

pub async fn update(cli: &Cli, doctype: &str, name: &str, file: &str) -> Result<Value, CliError> {
    let value = read_json_input(file)?;
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let resp = client.update_doc(doctype, name, &value).await?;
    let v: Value = serde_json::from_str(&resp)?;
    Ok(json!({"ok": true, "data": v}))
}

pub async fn delete(cli: &Cli, doctype: &str, name: &str, yes: bool) -> Result<Value, CliError> {
    if !yes {
        use std::io::BufRead;
        eprint!("Delete {doctype} `{name}`? [y/N] ");
        std::io::stderr().flush().ok();
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        if !matches!(line.trim().to_lowercase().as_str(), "y" | "yes") {
            return Err(CliError::Aborted);
        }
    }
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    client.delete_doc(doctype, name).await?;
    Ok(json!({"ok": true, "message": format!("Deleted {doctype} `{name}`")}))
}
