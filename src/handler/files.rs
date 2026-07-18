use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::PathBuf;

pub async fn handle(cli: &Cli, number: &str) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let pengajuan_name = if number.starts_with("ODR-") {
        number.to_string()
    } else {
        let result = client
            .doctype("Master Data Nasabah")
            .filter("no_perjanjian", "=", number)
            .fields(vec!["name", "register_id"])
            .limit(1)
            .execute_raw()
            .await?;
        let v: Value = serde_json::from_str(&result)?;

        let register_id = v["data"][0]["register_id"].as_str().ok_or_else(|| {
            CliError::Other("Agreement number not found or has no register_id".into())
        })?;

        if register_id.is_empty() {
            return Err(CliError::Other(
                "register_id is empty; cannot find Data Pengajuan".into(),
            ));
        }
        register_id.to_string()
    };

    let body = client.get_doc("Data Pengajuan", &pengajuan_name).await?;
    let data: Value = serde_json::from_str(&body)?;

    let files = collect_files(&data);

    if files.is_empty() {
        eprintln!("No attachment files found in this document.");
        return Ok(json!({ "downloaded": [] }));
    }

    eprintln!("Attached files in Data Pengajuan `{pengajuan_name}`:");
    for (i, (label, url)) in files.iter().enumerate() {
        eprintln!("  {}. {} -> {}", i + 1, label, url);
    }

    eprint!("\nDownload? [Y/n/1,2,3]: ");
    std::io::stderr().flush().ok();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    let choice = line.trim().to_lowercase();

    if choice == "n" || choice == "no" {
        return Ok(json!({ "downloaded": [], "message": "Cancelled" }));
    }

    let selected: Vec<usize> = if choice == "y" || choice == "yes" || choice.is_empty() {
        (0..files.len()).collect()
    } else {
        choice
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok().map(|n| n - 1))
            .filter(|&n| n < files.len())
            .collect()
    };

    if selected.is_empty() {
        eprintln!("No files selected.");
        return Ok(json!({ "downloaded": [] }));
    }

    let download_dir = dirs_next::download_dir().unwrap_or_else(|| PathBuf::from("."));

    let mut downloaded = Vec::new();
    for idx in selected {
        let (label, url) = &files[idx];
        let filename = url.rsplit('/').next().unwrap_or("file");
        let save_path = download_dir.join(filename);
        eprintln!("Downloading `{}` to {}...", label, save_path.display());
        client.download_file_to_path(url, &save_path).await?;
        downloaded.push(json!({
            "label": label,
            "url": url,
            "saved_to": save_path.display().to_string(),
        }));
    }

    Ok(json!({ "downloaded": downloaded }))
}

fn collect_files(value: &Value) -> Vec<(String, String)> {
    let mut files = Vec::new();

    if let Value::Object(obj) = value
        && let Some(data) = obj.get("data")
        && let Value::Object(data_obj) = data
    {
        for (key, val) in data_obj {
            if let Value::String(s) = val
                && (s.starts_with("/private/files/") || s.starts_with("/files/"))
            {
                files.push((key.clone(), s.clone()));
            }
        }
    }

    if files.is_empty()
        && let Value::Object(root) = value
    {
        for (key, val) in root {
            if let Value::String(s) = val
                && (s.starts_with("/private/files/") || s.starts_with("/files/"))
            {
                files.push((key.clone(), s.clone()));
            }
        }
    }

    files
}
