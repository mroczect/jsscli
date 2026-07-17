use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::{CliError, CliResult};
use crate::output;
use std::io::Read;

fn read_json_input(file: &str) -> CliResult<serde_json::Value> {
    let raw = if file == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        let expanded = shellexpand::tilde(file).to_string();
        std::fs::read_to_string(&expanded)?
    };
    let v: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| CliError::Usage(format!("Invalid JSON in `{file}`: {e}")))?;
    Ok(v)
}

pub async fn get(cli: &Cli, doctype: &str, name: &str) -> CliResult<()> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let body = client.get_doc(doctype, name).await?;
    output::print_string(&body, cli.output)?;
    Ok(())
}

pub async fn list(
    cli: &Cli,
    doctype: &str,
    fields: Option<Vec<String>>,
    filters: Vec<String>,
    limit: u32,
    start: u32,
    order_by: Option<String>,
) -> CliResult<()> {
    if doctype.is_empty() {
        return Err(CliError::Usage("doctype is required".into()));
    }
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let mut builder = client.doctype(doctype).limit(limit).limit_start(start);
    if let Some(fields) = fields {
        let refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
        builder = builder.fields(refs);
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
    output::print_string(&raw, cli.output)?;
    Ok(())
}

pub async fn create(cli: &Cli, doctype: &str, file: &str) -> CliResult<()> {
    let value = read_json_input(file)?;
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let resp = client.create_doc(doctype, &value).await?;
    output::print_string(&resp, cli.output)?;
    if !cli.quiet {
        output::success(format!("Created {doctype} document"));
    }
    Ok(())
}

pub async fn update(cli: &Cli, doctype: &str, name: &str, file: &str) -> CliResult<()> {
    let value = read_json_input(file)?;
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let resp = client.update_doc(doctype, name, &value).await?;
    output::print_string(&resp, cli.output)?;
    if !cli.quiet {
        output::success(format!("Updated {doctype} `{name}`"));
    }
    Ok(())
}

pub async fn delete(cli: &Cli, doctype: &str, name: &str, yes: bool) -> CliResult<()> {
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
    let _ = client.delete_doc(doctype, name).await?;
    if !cli.quiet {
        output::success(format!("Deleted {doctype} `{name}`"));
    }
    Ok(())
}

use std::io::Write;
