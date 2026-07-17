use crate::error::{CliError, CliResult};
use chrono::{DateTime, Utc};
use librjss::handler::types::SessionInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) mod crypto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub account: String,
    pub full_name: Option<String>,
    pub sitename: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

impl SessionRecord {
    pub fn from_info(account: &str, info: &SessionInfo) -> Self {
        let now = Utc::now();
        Self {
            account: account.to_string(),
            full_name: info.full_name.clone(),
            sitename: info.sitename.clone(),
            roles: info.roles.clone(),
            created_at: now,
            last_used_at: now,
        }
    }

    pub fn path() -> CliResult<PathBuf> {
        Ok(dirs_next::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("jsscli")
            .join("session.enc"))
    }

    pub fn save(&self) -> CliResult<()> {
        let p = Self::path()?;
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let key = self::crypto::load_or_create_key()
            .map_err(|e| CliError::Config(format!("Encryption key error: {e}")))?;
        let encrypted = crate::crypto::encrypt(&key, &json)
            .map_err(|e| CliError::Config(format!("Encryption failed: {e}")))?;
        std::fs::write(&p, encrypted)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    pub fn load() -> CliResult<Option<Self>> {
        let p = Self::path()?;
        if !p.exists() {
            return Ok(None);
        }
        let key = self::crypto::load_or_create_key()
            .map_err(|e| CliError::Config(format!("Encryption key error: {e}")))?;
        let encrypted = std::fs::read_to_string(&p)?;
        let json =
            crate::crypto::decrypt(&key, &encrypted).map_err(|_| CliError::SessionExpired)?;
        let rec: SessionRecord = serde_json::from_str(&json)?;
        Ok(Some(rec))
    }

    pub fn clear() -> CliResult<()> {
        let p = Self::path()?;
        if p.exists() {
            std::fs::remove_file(&p)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn exists() -> bool {
        Self::path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn matches_account(&self, account: &str) -> bool {
        self.account == account
    }
}
