use crate::config::{Account, AccountStore, Settings};
use crate::error::{CliError, CliResult};
use librjss::RjssClient;
use librjss::handler::config::{AuthMode, ClientConfig};

pub fn resolve_active_account_name(
    settings: &Settings,
    profile: Option<&str>,
) -> CliResult<String> {
    settings.resolve_active(profile)
}

pub fn load_active_account(
    settings: &Settings,
    profile: Option<&str>,
) -> CliResult<(String, Account)> {
    let name = resolve_active_account_name(settings, profile)?;
    let store = AccountStore::load()?;
    let acc = store.get(&name)?.clone();
    Ok((name, acc))
}

pub fn build_client_for(account: &Account) -> CliResult<RjssClient> {
    let base_url = reqwest::Url::parse(&account.base_url)
        .map_err(|e| CliError::Config(format!("Invalid base_url for account: {e}")))?;

    let auth_mode: AuthMode = account.to_auth_mode()?;

    let cfg = ClientConfig {
        base_url,
        auth_mode,
        expected_sitename: account.expected_sitename.clone(),
        required_roles: account.required_roles.clone(),
        timeout_secs: account.timeout_secs,
        max_retries: account.max_retries,
        user_agent: account.user_agent.clone(),
        insecure_ssl: account.insecure_ssl,
    };

    RjssClient::new(cfg).map_err(CliError::from)
}

pub async fn create_client(
    settings: &Settings,
    profile: Option<&str>,
) -> CliResult<(RjssClient, String)> {
    let (name, account) = load_active_account(settings, profile)?;
    let mut client = build_client_for(&account)?;
    client.authenticate().await.map_err(CliError::Jss)?;
    Ok((client, name))
}
