mod cli;
mod client;
mod command;
mod config;
mod crypto;
mod error;
mod handler;
mod output;
mod session;
use clap::Parser;
use cli::Cli;

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,jsscli=info,librjss=warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    let cli = Cli::parse();

    match command::dispatch(cli).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}", error::format_error(&e));
            std::process::exit(match e {
                error::CliError::Usage(_) => 2,
                error::CliError::NotAuthenticated => 3,
                error::CliError::Network(_) => 4,
                error::CliError::Api(_) => 5,
                _ => 1,
            });
        }
    }
}
