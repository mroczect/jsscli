use crate::cli::Cli;
use crate::error::CliError;
use minus::{Pager, page_all};
use serde_json::{Value, json};

pub async fn handle(_cli: &Cli) -> Result<Value, CliError> {
    let manual_content = include_str!("../../manual/jsscli.0.1.0.txt");

    let pager = Pager::new();
    pager
        .set_text(manual_content)
        .map_err(|e| CliError::Other(format!("Failed to set pager text: {e}")))?;
    pager
        .set_prompt("jsscli manual - q to quit, arrows to scroll")
        .map_err(|e| CliError::Other(format!("Failed to set pager prompt: {e}")))?;

    page_all(pager).map_err(|e| CliError::Other(format!("Pager error: {e}")))?;

    Ok(json!({"status": "Manual displayed"}))
}
