use crate::cli::Cli;
use crate::error::CliResult;

pub async fn handle(_cli: &Cli) -> CliResult<()> {
    let manual_content = include_str!("../../manual/jsscli.0.1.0.txt");
    println!("{manual_content}");
    Ok(())
}
