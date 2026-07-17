use crate::cli::Cli;
use crate::error::CliResult;
use crate::output;
use serde_json::{json, Value};

pub async fn handle(count: usize, cli: &Cli) -> CliResult<()> {
    // Baca dari environment variable (diisi saat kompilasi via build.rs)
    let changelog_raw = std::env::var("CHANGELOG_JSON")
        .unwrap_or_else(|_| "[]".to_string());

    let mut changelog: Vec<Value> = serde_json::from_str(&changelog_raw)
        .unwrap_or_else(|_| vec![]);

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
