use crate::cli::{Cli, DoctypeCommand};
use crate::error::CliResult;
use crate::handler;

pub async fn handle(cmd: DoctypeCommand, cli: &Cli) -> CliResult<()> {
    match cmd {
        DoctypeCommand::Get { doctype, name } => handler::doctype::get(cli, &doctype, &name).await,
        DoctypeCommand::List {
            doctype,
            fields,
            filter,
            limit,
            start,
            order_by,
        } => {
            handler::doctype::list(
                cli,
                &doctype,
                fields,
                filter.unwrap_or_default(),
                limit,
                start,
                order_by,
            )
            .await
        }
        DoctypeCommand::Create { doctype, file } => {
            handler::doctype::create(cli, &doctype, &file).await
        }
        DoctypeCommand::Update {
            doctype,
            name,
            file,
        } => handler::doctype::update(cli, &doctype, &name, &file).await,
        DoctypeCommand::Delete { doctype, name, yes } => {
            handler::doctype::delete(cli, &doctype, &name, yes).await
        }
    }
}
