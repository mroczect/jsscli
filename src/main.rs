mod cli;
mod client;
mod command;
mod config;
mod crypto;
mod error;
mod handler;
mod session;

use clap::Parser;
use cli::Cli;
use serde_json::json;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match command::dispatch(cli).await {
        Ok(value) => {
            println!("{}", serde_json::to_string(&value).unwrap());
        }
        Err(e) => {
            let error_json = json!({
                "ok": false,
                "error": error::format_error(&e),
                "code": error::exit_code(&e),
            });
            eprintln!("{}", serde_json::to_string(&error_json).unwrap());
            std::process::exit(error::exit_code(&e));
        }
    }
}
