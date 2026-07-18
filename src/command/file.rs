use crate::cli::{Cli, FileCommand};
use crate::error::CliError;
use crate::handler;
use serde_json::Value;

pub async fn handle(cmd: FileCommand, cli: &Cli) -> Result<Value, CliError> {
    match cmd {
        FileCommand::Upload {
            file,
            doctype,
            docname,
            fieldname,
        } => handler::file::upload(cli, &file, &doctype, &docname, &fieldname).await,
        FileCommand::Download { url, output, yes } => {
            handler::file::download(cli, &url, output, yes).await
        }
        FileCommand::List {
            folder,
            limit,
            search,
            file_type,
            attached_to_doctype,
            attached_to_name,
        } => {
            handler::file::list_files(
                cli,
                &folder,
                limit,
                search.as_deref(),
                file_type.as_deref(),
                attached_to_doctype.as_deref(),
                attached_to_name.as_deref(),
            )
            .await
        }
    }
}
