use crate::cli::{AccountCommand, Cli};
use crate::error::CliResult;
use crate::handler;

pub async fn handle(cmd: AccountCommand, cli: &Cli) -> CliResult<()> {
    match cmd {
        AccountCommand::Add {
            name,
            url,
            mode,
            email,
            password,
            api_key,
            api_secret,
            sitename,
            activate,
        } => {
            handler::account::add(
                cli, name, url, mode, email, password, api_key, api_secret, sitename, activate,
            )
            .await
        }
        AccountCommand::List => handler::account::list(cli).await,
        AccountCommand::Use { name } => handler::account::use_account(cli, name).await,
        AccountCommand::Remove { name } => handler::account::remove(cli, name).await,
        AccountCommand::Show { name } => handler::account::show(cli, name).await,
    }
}
