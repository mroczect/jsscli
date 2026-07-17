use crate::cli::{AuthModeKind, Cli};
use crate::config::{
    Account, AccountStore, AuthModeSer, Settings,
    accounts::{delete_secret, read_password, set_secret},
};
use crate::error::{CliError, CliResult};
use crate::output;
use crate::session::SessionRecord;

pub async fn add(
    cli: &Cli,
    name: String,
    url: Option<String>,
    mode: AuthModeKind,
    email: Option<String>,
    password: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    sitename: Option<String>,
    activate: bool,
) -> CliResult<()> {
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
        if !cli.quiet {
            output::success(format!("Account `{name}` added and activated"));
        }
    } else if !cli.quiet {
        output::success(format!("Account `{name}` added (not active)"));
    }
    Ok(())
}

pub async fn list(cli: &Cli) -> CliResult<()> {
    let settings = Settings::load()?;
    let store = AccountStore::load()?;

    if store.accounts.is_empty() {
        if !cli.quiet {
            output::warn("No accounts configured. Run `jsscli account add <name> --url ...`.");
        }
        return Ok(());
    }

    let active = settings.active_account.as_deref().unwrap_or("");
    let rows: Vec<serde_json::Value> = store
        .list()
        .into_iter()
        .map(|a| {
            let is_active = a.name == active;
            serde_json::json!({
                "active": if is_active { "*" } else { "" },
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

    let v = serde_json::Value::Array(rows);
    crate::output::print_data(&v, cli.output)?;
    Ok(())
}

pub async fn use_account(cli: &Cli, name: String) -> CliResult<()> {
    let store = AccountStore::load()?;
    let _ = store.get(&name)?;
    let mut settings = Settings::load()?;
    settings.active_account = Some(name.clone());
    settings.save()?;
    if !cli.quiet {
        output::success(format!("Switched active account to `{name}`"));
    }
    Ok(())
}

pub async fn remove(cli: &Cli, name: String) -> CliResult<()> {
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
    if !cli.quiet {
        output::success(format!("Removed account `{}`", removed.name));
    }
    Ok(())
}

pub async fn show(cli: &Cli, name: Option<String>) -> CliResult<()> {
    let store = AccountStore::load()?;
    let settings = Settings::load()?;
    let target = name.unwrap_or_else(|| settings.active_account.clone().unwrap_or_default());
    let account = store.get(&target)?;
    let pairs = account.masked_summary();
    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(k, serde_json::Value::String(v));
    }
    let v = serde_json::Value::Object(map);
    crate::output::print_data(&v, cli.output)?;
    Ok(())
}
