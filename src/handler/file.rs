use crate::cli::Cli;
use crate::client;
use crate::config::Settings;
use crate::error::{CliError, CliResult};
use crate::output;
use std::io::Write;
use std::path::{Path, PathBuf};

pub async fn upload(
    cli: &Cli,
    file: &str,
    doctype: &str,
    docname: &str,
    fieldname: &str,
) -> CliResult<()> {
    let expanded = shellexpand::tilde(file).to_string();
    let path = Path::new(&expanded);
    if !path.exists() {
        return Err(CliError::Usage(format!("File not found: {file}")));
    }
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CliError::Usage("Cannot determine file name".into()))?;

    const MAX_BYTES: u64 = 50 * 1024 * 1024;
    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_BYTES {
        return Err(CliError::Usage(format!(
            "File too large ({} bytes; max {MAX_BYTES})",
            meta.len()
        )));
    }

    let content = std::fs::read(path)?;

    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;
    let resp = client
        .upload_file(file_name, content, doctype, docname, fieldname)
        .await?;
    output::print_string(&resp, cli.output)?;
    if !cli.quiet {
        output::success(format!(
            "Uploaded `{file_name}` → {doctype}/{docname}/{fieldname}"
        ));
    }
    Ok(())
}

pub async fn download(
    cli: &Cli,
    url: &str,
    output_path: Option<String>,
    yes: bool,
) -> CliResult<()> {
    let settings = Settings::load()?;
    let (client, _) = client::create_client(&settings, cli.profile.as_deref()).await?;

    let out: PathBuf = match output_path {
        Some(p) => shellexpand::tilde(&p).to_string().into(),
        None => {
            let basename = url
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("downloaded_file")
                .to_string();
            PathBuf::from(basename)
        }
    };

    if out.exists() && !yes {
        use std::io::BufRead;
        eprint!("File `{}` exists. Overwrite? [y/N] ", out.display());
        std::io::stderr().flush().ok();
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        if !matches!(line.trim().to_lowercase().as_str(), "y" | "yes") {
            return Err(CliError::Aborted);
        }
    }

    client.download_file_to_path(url, &out).await?;
    if !cli.quiet {
        output::success(format!("Saved `{}`", out.display()));
    }
    Ok(())
}
