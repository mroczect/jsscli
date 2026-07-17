use crate::error::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub active_account: Option<String>,
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
}

impl Settings {
    pub fn path() -> CliResult<PathBuf> {
        let base = dirs_next::config_dir()
            .ok_or_else(|| CliError::Config("Cannot determine config dir".into()))?;
        Ok(base.join("jsscli").join("config.toml"))
    }

    pub fn load() -> CliResult<Self> {
        let p = Self::path()?;
        if !p.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(&p)?;
        let mut settings: Settings = toml::from_str(&s)?;
        if let Ok(name) = std::env::var("JSS_ACTIVE_ACCOUNT") {
            settings.active_account = Some(name);
        }
        Ok(settings)
    }

    pub fn save(&self) -> CliResult<()> {
        let p = Self::path()?;
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self)?;
        std::fs::write(&p, s)?;
        Ok(())
    }

    pub fn ensure_dirs() -> CliResult<()> {
        let p = Self::path()?;
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> CliResult<()> {
        match key {
            "active_account" => self.active_account = Some(value.to_string()),
            other => {
                self.defaults.insert(other.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "active_account" => self.active_account.clone(),
            other => self.defaults.get(other).cloned(),
        }
    }

    pub fn list_pairs(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        out.push((
            "active_account".into(),
            self.active_account.clone().unwrap_or_default(),
        ));
        for (k, v) in &self.defaults {
            out.push((k.clone(), v.clone()));
        }
        out
    }

    pub fn resolve_active(&self, profile_override: Option<&str>) -> CliResult<String> {
        if let Some(p) = profile_override {
            return Ok(p.to_string());
        }
        self.active_account.clone().ok_or_else(|| {
            CliError::Config("No active account. Run `jsscli account use <name>`.".into())
        })
    }
}

#[allow(dead_code)]
pub fn pretty_print_path(p: &Path) -> String {
    p.display().to_string()
}
