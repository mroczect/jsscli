use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::{CliError, CliResult};
use crate::output;
use crate::session::SessionRecord;

pub async fn login(cli: &Cli) -> CliResult<()> {
    let settings = Settings::load()?;
    let (client, account_name) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let info = client.session_info().ok_or(CliError::Other(
        "Login succeeded but no session info returned".into(),
    ))?;

    let rec = SessionRecord::from_info(&account_name, info);
    rec.save()?;

    if !cli.quiet {
        output::success(format!(
            "Logged in as {} on site `{}`",
            info.full_name.as_deref().unwrap_or("(unknown)"),
            info.sitename,
        ));
        if !info.roles.is_empty() {
            output::info(format!("Roles: {}", info.roles.join(", ")));
        }
    }
    Ok(())
}

pub async fn logout(cli: &Cli) -> CliResult<()> {
    let settings = Settings::load()?;
    let result = if let Ok((mut client, _)) =
        client::create_client(&settings, cli.profile.as_deref()).await
    {
        client.logout().await
    } else {
        Ok(())
    };
    SessionRecord::clear()?;
    if !cli.quiet {
        output::success("Logged out");
    }
    if let Err(e) = result {
        output::warn(format!("Server-side logout returned an error: {e}"));
    }
    Ok(())
}

pub async fn status(_cli: &Cli) -> CliResult<()> {
    let settings = Settings::load()?;
    let active = settings
        .active_account
        .clone()
        .unwrap_or_else(|| "(none)".into());

    match SessionRecord::load()? {
        Some(rec) => {
            let valid = rec.matches_account(&active);
            if valid {
                output::success(format!(
                    "Authenticated as `{}` on `{}` (account: {})",
                    rec.full_name.as_deref().unwrap_or("(unknown)"),
                    rec.sitename,
                    rec.account,
                ));
            } else {
                output::warn(format!(
                    "Session exists for account `{}` but active account is `{}`.",
                    rec.account, active
                ));
            }
        }
        None => {
            output::warn(format!("Not authenticated (active account: {active})"));
        }
    }
    Ok(())
}

pub async fn whoami(cli: &Cli) -> CliResult<()> {
    let settings = Settings::load()?;
    let active = settings.resolve_active(cli.profile.as_deref())?;

    if let Some(rec) = SessionRecord::load()? {
        if rec.matches_account(&active) {
            let json = serde_json::json!({
                "full_name": rec.full_name,
                "sitename": rec.sitename,
                "roles": rec.roles,
            });
            crate::output::print_data(&json, cli.output)?;
            return Ok(());
        }
    }

    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    match client.session_info() {
        Some(info) => {
            let json = serde_json::json!({
                "full_name": info.full_name,
                "sitename": info.sitename,
                "roles": info.roles,
            });
            crate::output::print_data(&json, cli.output)?;
            Ok(())
        }
        None => Err(CliError::NotAuthenticated),
    }
}
