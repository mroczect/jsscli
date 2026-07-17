use crate::cli::Cli;
use crate::error::CliResult;
use crate::output;
use serde_json::{Value, json};

pub async fn handle(count: usize, cli: &Cli) -> CliResult<()> {
	let changelog_raw = option_env!("CHANGELOG_JSON").unwrap_or("[]");

    let mut changelog: Vec<Value> = serde_json::from_str(&changelog_raw).unwrap_or_else(|_| vec![]);

    if count < changelog.len() {
        changelog.truncate(count);
    }

    let output_data = json!({
        "total_commits": changelog.len(),
        "commits": changelog,
    });

    output::print_data(&output_data, cli.output)?;
    Ok(())
}
