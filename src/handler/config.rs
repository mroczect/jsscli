use crate::cli::Cli;
use crate::config::Settings;
use crate::error::CliError;
use serde_json::{Value, json};

pub async fn set(_cli: &Cli, key: String, value: String) -> Result<Value, CliError> {
    let mut settings = Settings::load()?;
    settings.set(&key, &value)?;
    settings.save()?;
    Ok(json!({"ok": true, "key": key, "value": value}))
}

pub async fn get(_cli: &Cli, key: String) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    match settings.get(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Err(CliError::Config(format!("Key `{key}` not found"))),
    }
}

pub async fn list(_cli: &Cli) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let pairs = settings.list_pairs();
    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(k, Value::String(v));
    }
    Ok(Value::Object(map))
}

pub async fn init(_cli: &Cli) -> Result<Value, CliError> {
    Settings::ensure_dirs()?;
    let _ = Settings::load()?;
    let _ = crate::config::AccountStore::load()?;
    let _ = crate::session::SessionRecord::path();
    Ok(json!({
        "ok": true,
        "config_path": Settings::path()?.display().to_string(),
        "accounts_path": crate::config::AccountStore::path()?.display().to_string(),
        "session_path": crate::session::SessionRecord::path()?.display().to_string(),
    }))
}

pub async fn path_cmd(_cli: &Cli) -> Result<Value, CliError> {
    Ok(Value::String(Settings::path()?.display().to_string()))
}
