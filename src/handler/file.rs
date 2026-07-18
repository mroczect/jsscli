use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub async fn upload(
    cli: &Cli,
    file: &str,
    doctype: &str,
    docname: &str,
    fieldname: &str,
) -> Result<Value, CliError> {
    let expanded = shellexpand::tilde(file).to_string();
    let path = Path::new(&expanded);
    if !path.exists() {
        return Err(CliError::Usage(format!("File not found: {file}")));
    }
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CliError::Usage("Cannot determine file name".into()))?;

    const MAX_BYTES: u64 = 50 * 1024 * 1024;
    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_BYTES {
        return Err(CliError::Usage(format!(
            "File too large ({} bytes; max {MAX_BYTES})",
            meta.len()
        )));
    }

    let content = std::fs::read(path)?;

    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let resp = client
        .upload_file(file_name, content, doctype, docname, fieldname)
        .await?;
    let v: Value = serde_json::from_str(&resp).unwrap_or(Value::String(resp));
    Ok(json!({"ok": true, "data": v}))
}

pub async fn download(
    cli: &Cli,
    url: &str,
    output_path: Option<String>,
    yes: bool,
) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let out: PathBuf = match output_path {
        Some(p) => shellexpand::tilde(&p).to_string().into(),
        None => {
            let basename = url
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("downloaded_file")
                .to_string();
            PathBuf::from(basename)
        }
    };

    if out.exists() && !yes {
        use std::io::BufRead;
        eprint!("File `{}` exists. Overwrite? [y/N] ", out.display());
        std::io::stderr().flush().ok();
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        if !matches!(line.trim().to_lowercase().as_str(), "y" | "yes") {
            return Err(CliError::Aborted);
        }
    }

    client.download_file_to_path(url, &out).await?;
    Ok(json!({"ok": true, "path": out.display().to_string()}))
}

pub async fn list_files(
    cli: &Cli,
    folder: &str,
    limit: u32,
    search: Option<&str>,
    file_type: Option<&str>,
    attached_to_doctype: Option<&str>,
    attached_to_name: Option<&str>,
) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let mut filters = vec![json!(["File", "folder", "=", folder])];

    if let Some(s) = search {
        filters.push(json!(["File", "file_name", "like", format!("%{}%", s)]));
    }
    if let Some(ft) = file_type {
        filters.push(json!(["File", "file_type", "like", format!("%{}%", ft)]));
    }
    if let Some(adt) = attached_to_doctype {
        filters.push(json!(["File", "attached_to_doctype", "=", adt]));
    }
    if let Some(adn) = attached_to_name {
        filters.push(json!(["File", "attached_to_name", "=", adn]));
    }

    let fields = json!([
        "`tabFile`.`file_name`",
        "`tabFile`.`file_url`",
        "`tabFile`.`file_size`",
        "`tabFile`.`file_type`",
        "`tabFile`.`is_private`",
        "`tabFile`.`attached_to_doctype`",
        "`tabFile`.`attached_to_name`",
        "`tabFile`.`attached_to_field`",
        "`tabFile`.`creation`",
        "`tabFile`.`modified`",
    ]);

    let mut params = HashMap::new();
    params.insert("doctype".to_string(), "File".to_string());
    params.insert("fields".to_string(), fields.to_string());
    params.insert(
        "filters".to_string(),
        serde_json::to_string(&filters).unwrap_or_default(),
    );
    params.insert(
        "order_by".to_string(),
        "`tabFile`.`creation` desc".to_string(),
    );
    params.insert("start".to_string(), "0".to_string());
    params.insert("page_length".to_string(), limit.to_string());
    params.insert("view".to_string(), "List".to_string());

    let resp = client
        .call_method("frappe.desk.reportview.get", Some(params))
        .await?;
    let mut v: Value = serde_json::from_str(&resp)?;

    if let Some(message) = v.get_mut("message")
        && let (Some(keys), Some(values)) = (message.get("keys"), message.get("values"))
        && let (Some(keys_arr), Some(values_arr)) = (keys.as_array(), values.as_array())
    {
        let mut items = Vec::new();
        for row in values_arr {
            if let Some(row_arr) = row.as_array() {
                let mut obj = serde_json::Map::new();
                for (i, key) in keys_arr.iter().enumerate() {
                    if let Some(key_str) = key.as_str() {
                        let val = row_arr.get(i).cloned().unwrap_or(Value::Null);
                        obj.insert(key_str.to_string(), val);
                    }
                }
                items.push(Value::Object(obj));
            }
        }
        *message = json!(items);
    }

    Ok(v)
}
