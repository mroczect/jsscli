use crate::cli::Cli;
use crate::error::CliResult;

pub async fn handle(count: usize, cli: &Cli) -> CliResult<()> {
    crate::handler::changelog::handle(count, cli).await
}
