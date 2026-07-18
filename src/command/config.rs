use crate::cli::{Cli, ConfigCommand};
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(cmd: ConfigCommand, cli: &Cli) -> Result<Value, CliError> {
    match cmd {
        ConfigCommand::Set { key, value } => handler::config::set(cli, key, value).await,
        ConfigCommand::Get { key } => handler::config::get(cli, key).await,
        ConfigCommand::List => handler::config::list(cli).await,
        ConfigCommand::Init => handler::config::init(cli).await,
        ConfigCommand::Path => handler::config::path_cmd(cli).await,
    }
}
