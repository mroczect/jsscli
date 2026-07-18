use crate::cli::Cli;
use crate::error::CliError;
use serde_json::{Value, json};

pub async fn handle(count: usize, _cli: &Cli) -> Result<Value, CliError> {
    let changelog_raw = option_env!("CHANGELOG_JSON").unwrap_or("[]");
    let mut changelog: Vec<Value> = serde_json::from_str(changelog_raw).unwrap_or_default();
    changelog.truncate(count);
    Ok(json!({
        "total_commits": changelog.len(),
        "commits": changelog,
    }))
}
