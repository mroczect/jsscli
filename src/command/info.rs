use crate::cli::Cli;
use crate::error::CliResult;

pub async fn handle(cli: &Cli) -> CliResult<()> {
    crate::handler::info::handle(cli).await
}
