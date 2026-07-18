use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::CliError;
use crate::session::SessionRecord;
use serde_json::{Value, json};

pub async fn login(cli: &Cli) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let (client, account_name) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let info = client.session_info().ok_or(CliError::Other(
        "Login succeeded but no session info returned".into(),
    ))?;

    let rec = SessionRecord::from_info(&account_name, info);
    rec.save()?;

    Ok(json!({
        "ok": true,
        "account": account_name,
        "full_name": info.full_name,
        "sitename": info.sitename,
        "roles": info.roles,
    }))
}

pub async fn logout(cli: &Cli) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let result = if let Ok((mut client, _)) =
        client::create_client(&settings, cli.profile.as_deref()).await
    {
        client.logout().await
    } else {
        Ok(())
    };
    SessionRecord::clear()?;
    if let Err(e) = result {
        return Err(CliError::Api(e.to_string()));
    }
    Ok(json!({"ok": true, "message": "Logged out"}))
}

pub async fn status(_cli: &Cli) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let active = settings
        .active_account
        .clone()
        .unwrap_or_else(|| "(none)".into());

    match SessionRecord::load()? {
        Some(rec) => {
            let valid = rec.matches_account(&active);
            Ok(json!({
                "authenticated": valid,
                "account": rec.account,
                "full_name": rec.full_name,
                "sitename": rec.sitename,
                "active_account": active,
            }))
        }
        None => Ok(json!({
            "authenticated": false,
            "active_account": active,
        })),
    }
}

pub async fn whoami(cli: &Cli) -> Result<Value, CliError> {
    let settings = Settings::load()?;
    let active = settings.resolve_active(cli.profile.as_deref())?;

    if let Some(rec) = SessionRecord::load()? {
        if rec.matches_account(&active) {
            return Ok(json!({
                "full_name": rec.full_name,
                "sitename": rec.sitename,
                "roles": rec.roles,
            }));
        }
    }

    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    match client.session_info() {
        Some(info) => Ok(json!({
            "full_name": info.full_name,
            "sitename": info.sitename,
            "roles": info.roles,
        })),
        None => Err(CliError::NotAuthenticated),
    }
}
