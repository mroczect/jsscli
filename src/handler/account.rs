use crate::cli::{AuthModeKind, Cli};
use crate::config::{
    Account, AccountStore, AuthModeSer, Settings,
    accounts::{delete_secret, read_password, set_secret},
};
use crate::error::CliError;
use crate::session::SessionRecord;
use serde_json::{Value, json};

pub async fn add(
    _cli: &Cli,
    name: String,
    url: Option<String>,
    mode: AuthModeKind,
    email: Option<String>,
    password: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    sitename: Option<String>,
    activate: bool,
) -> Result<Value, CliError> {
    let url = url.ok_or_else(|| CliError::Usage("--url is required".into()))?;

    let auth_mode = match mode {
        AuthModeKind::Session => {
            let email = email
                .ok_or_else(|| CliError::Usage("--email is required for session mode".into()))?;
            if email.is_empty() {
                return Err(CliError::Usage("Email must not be empty".into()));
            }
            let password = match password {
                Some(p) if !p.is_empty() => p,
                _ => read_password("Password: ")?,
            };
            set_secret(&name, "password", &password)?;
            AuthModeSer::Session { email }
        }
        AuthModeKind::Token => {
            let api_key = api_key
                .ok_or_else(|| CliError::Usage("--api-key is required for token mode".into()))?;
            if api_key.is_empty() {
                return Err(CliError::Usage("API Key must not be empty".into()));
            }
            let api_secret = api_secret
                .ok_or_else(|| CliError::Usage("--api-secret is required for token mode".into()))?;
            if api_secret.is_empty() {
                return Err(CliError::Usage("API Secret must not be empty".into()));
            }
            set_secret(&name, "api_secret", &api_secret)?;
            AuthModeSer::Token { api_key }
        }
    };

    let mut store = AccountStore::load()?;
    if store.accounts.contains_key(&name) {
        return Err(CliError::Usage(format!(
            "Account `{name}` already exists. Remove it first."
        )));
    }

    let account = Account {
        name: name.clone(),
        base_url: url,
        mode: auth_mode,
        expected_sitename: sitename,
        required_roles: Vec::new(),
        timeout_secs: 30,
        max_retries: 3,
        user_agent: "jsscli/0.3".into(),
        insecure_ssl: false,
    };
    store.upsert(account);
    store.save()?;

    if activate {
        let mut settings = Settings::load()?;
        settings.active_account = Some(name.clone());
        settings.save()?;
    }

    Ok(json!({
        "ok": true,
        "name": name,
        "active": activate
    }))
}

pub async fn list(_cli: &Cli) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let store = AccountStore::load()?;

    let accounts: Vec<Value> = store
        .list()
        .into_iter()
        .map(|a| {
            let is_active = Some(a.name.as_str()) == settings.active_account.as_deref();
            json!({
                "active": is_active,
                "name": a.name,
                "base_url": a.base_url,
                "mode": match &a.mode {
                    AuthModeSer::Session { .. } => "session",
                    AuthModeSer::Token { .. } => "token",
                },
                "sitename": a.expected_sitename.clone().unwrap_or_default(),
            })
        })
        .collect();

    Ok(json!({ "accounts": accounts }))
}

pub async fn use_account(_cli: &Cli, name: String) -> Result<Value, CliError> {
    let store = AccountStore::load()?;
    let _ = store.get(&name)?;
    let mut settings = Settings::load()?;
    settings.active_account = Some(name.clone());
    settings.save()?;
    Ok(json!({"ok": true, "active_account": name}))
}

pub async fn remove(_cli: &Cli, name: String) -> Result<Value, CliError> {
    let mut store = AccountStore::load()?;
    let removed = store.remove(&name)?;
    store.save()?;

    match removed.mode {
        AuthModeSer::Session { .. } => delete_secret(&name, "password")?,
        AuthModeSer::Token { .. } => delete_secret(&name, "api_secret")?,
    }

    let mut settings = Settings::load()?;
    if settings.active_account.as_deref() == Some(name.as_str()) {
        settings.active_account = None;
        settings.save()?;
    }
    if let Some(rec) = SessionRecord::load()? {
        if rec.matches_account(&name) {
            let _ = SessionRecord::clear();
        }
    }
    Ok(json!({"ok": true, "removed": name}))
}

pub async fn show(_cli: &Cli, name: Option<String>) -> Result<Value, CliError> {
    let store = AccountStore::load()?;
    let settings = Settings::load()?;
    let target = name.unwrap_or_else(|| settings.active_account.clone().unwrap_or_default());
    let account = store.get(&target)?;
    let mut map = serde_json::Map::new();
    for (k, v) in account.masked_summary() {
        map.insert(k, Value::String(v));
    }
    Ok(Value::Object(map))
}
