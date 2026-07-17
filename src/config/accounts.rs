use crate::error::{CliError, CliResult};
use librjss::handler::config::AuthMode;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub mode: AuthModeSer,
    #[serde(default)]
    pub expected_sitename: Option<String>,
    #[serde(default)]
    pub required_roles: Vec<String>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_retries")]
    pub max_retries: u32,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default)]
    pub insecure_ssl: bool,
}

fn default_timeout() -> u64 {
    30
}
fn default_retries() -> u32 {
    3
}
fn default_user_agent() -> String {
    "jsscli/0.3".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AuthModeSer {
    Session { email: String },
    Token { api_key: String },
}

impl Default for AuthModeSer {
    fn default() -> Self {
        AuthModeSer::Session {
            email: String::new(),
        }
    }
}

pub fn get_secret(account_name: &str, secret_type: &str) -> CliResult<String> {
    let entry = keyring::Entry::new("jsscli", &format!("{account_name}-{secret_type}"))
        .map_err(|e| CliError::Config(format!("Keyring init error: {e}")))?;
    entry.get_password().map_err(|e| {
        CliError::Config(format!(
            "Failed to read secret '{secret_type}' from keyring: {e}"
        ))
    })
}

pub fn set_secret(account_name: &str, secret_type: &str, value: &str) -> CliResult<()> {
    let entry = keyring::Entry::new("jsscli", &format!("{account_name}-{secret_type}"))
        .map_err(|e| CliError::Config(format!("Keyring init error: {e}")))?;
    entry.set_password(value).map_err(|e| {
        CliError::Config(format!(
            "Failed to save secret '{secret_type}' to keyring: {e}"
        ))
    })
}

pub fn delete_secret(account_name: &str, secret_type: &str) -> CliResult<()> {
    let entry = keyring::Entry::new("jsscli", &format!("{account_name}-{secret_type}"))
        .map_err(|e| CliError::Config(format!("Keyring init error: {e}")))?;
    let _ = entry.delete_password();
    Ok(())
}

impl Account {
    pub fn to_auth_mode(&self) -> CliResult<AuthMode> {
        match &self.mode {
            AuthModeSer::Session { email } => {
                let password = get_secret(&self.name, "password")?;
                Ok(AuthMode::Session {
                    email: SecretString::from(email.as_str()),
                    password: SecretString::from(password.as_str()),
                })
            }
            AuthModeSer::Token { api_key } => {
                let api_secret = get_secret(&self.name, "api_secret")?;
                Ok(AuthMode::Token {
                    api_key: api_key.clone(),
                    api_secret: SecretString::from(api_secret.as_str()),
                })
            }
        }
    }

    pub fn masked_summary(&self) -> Vec<(String, String)> {
        let mut out = vec![
            ("name".into(), self.name.clone()),
            ("base_url".into(), self.base_url.clone()),
            (
                "expected_sitename".into(),
                self.expected_sitename.clone().unwrap_or_default(),
            ),
            ("timeout_secs".into(), self.timeout_secs.to_string()),
            ("max_retries".into(), self.max_retries.to_string()),
            ("insecure_ssl".into(), self.insecure_ssl.to_string()),
        ];
        match &self.mode {
            AuthModeSer::Session { email } => {
                out.push(("mode".into(), "session".into()));
                out.push(("email".into(), email.clone()));
                out.push(("password".into(), "******** (in keyring)".into()));
            }
            AuthModeSer::Token { api_key } => {
                out.push(("mode".into(), "token".into()));
                out.push(("api_key".into(), api_key.clone()));
                out.push(("api_secret".into(), "******** (in keyring)".into()));
            }
        }
        out
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountStore {
    #[serde(default)]
    pub accounts: BTreeMap<String, Account>,
}

impl AccountStore {
    pub fn path() -> CliResult<PathBuf> {
        let base = dirs_next::config_dir()
            .ok_or_else(|| CliError::Config("Cannot determine config dir".into()))?;
        Ok(base.join("jsscli").join("accounts.toml"))
    }

    pub fn load() -> CliResult<Self> {
        let p = Self::path()?;
        if !p.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(&p)?;
        let store: AccountStore = toml::from_str(&s)?;
        Ok(store)
    }

    pub fn save(&self) -> CliResult<()> {
        let p = Self::path()?;
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(&p, &s)?;
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
            return Ok(());
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&p, s)?;
            Ok(())
        }
    }

    pub fn get(&self, name: &str) -> CliResult<&Account> {
        self.accounts
            .get(name)
            .ok_or_else(|| CliError::AccountNotFound(name.to_string()))
    }

    pub fn upsert(&mut self, account: Account) {
        self.accounts.insert(account.name.clone(), account);
    }

    pub fn remove(&mut self, name: &str) -> CliResult<Account> {
        self.accounts
            .remove(name)
            .ok_or_else(|| CliError::AccountNotFound(name.to_string()))
    }

    pub fn list(&self) -> Vec<&Account> {
        self.accounts.values().collect()
    }
}

pub fn read_password(prompt: &str) -> CliResult<String> {
    let pass = rpassword::prompt_password(prompt)?;
    if pass.is_empty() {
        return Err(CliError::Usage("Password must not be empty".into()));
    }
    Ok(pass)
}
