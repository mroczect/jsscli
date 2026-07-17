use crate::cli::Cli;
use crate::error::CliResult;
use crate::output;
use serde_json::json;

pub async fn handle(cli: &Cli) -> CliResult<()> {
    let authors_raw = env!("CARGO_PKG_AUTHORS");
    let (first_author_name, first_author_email) = parse_first_author(authors_raw);

    let info = json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "authors": authors_raw,
        "author_name": first_author_name,
        "author_email": first_author_email,
        "homepage": env!("CARGO_PKG_HOMEPAGE"),
        "documentation": env!("CARGO_PKG_HOMEPAGE"),
        "license": env!("CARGO_PKG_LICENSE"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "build": {
            "hash": env!("BUILD_HASH"),
            "profile": env!("PROFILE"),
            "target": format!("{}-{}-{}", env!("TARGET_ARCH"), env!("TARGET_OS"), env!("TARGET_ENV")),
            "local_time": env!("BUILD_TIMESTAMP_LOCAL"),
            "utc_time": env!("BUILD_TIMESTAMP_UTC"),
            "hostname": env!("BUILD_HOSTNAME"),
        },
        "git": {
            "short_hash": env!("GIT_HASH"),
            "full_hash": env!("GIT_FULL_HASH"),
            "branch": env!("GIT_BRANCH"),
            "status": env!("GIT_DIRTY"),
            "commit_count": env!("GIT_COMMIT_COUNT"),
            "remote_url": env!("GIT_REMOTE_URL"),
        },
        "compiler": {
            "rustc": env!("RUSTC_VERSION"),
            "cargo": env!("CARGO_VERSION"),
        }
    });

    output::print_data(&info, cli.output)?;
    Ok(())
}

fn parse_first_author(authors: &str) -> (String, String) {
    let first = authors.split(':').next().unwrap_or(authors).trim();
    if let Some(pos) = first.find('<') {
        let name = first[..pos].trim();
        let rest = &first[pos + 1..];
        if let Some(end) = rest.find('>') {
            let email = rest[..end].trim();
            return (name.to_string(), email.to_string());
        }
    }
    (first.to_string(), "".to_string())
}
