use librjss::handler::error::JssError;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("Usage error: {0}")]
    Usage(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Not authenticated. Run `jsscli auth login` first.")]
    NotAuthenticated,
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Session expired or invalid")]
    SessionExpired,
    #[error("Operation aborted by user")]
    Aborted,
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    Jss(#[from] JssError),
    #[error("Data parsing error: {0}")]
    Parse(String),
}

pub type CliResult<T> = Result<T, CliError>;

pub fn format_error(err: &CliError) -> String {
    let mut out = format!("{err}");

    let mut source = std::error::Error::source(err);
    while let Some(s) = source {
        out.push_str(&format!("\n  → {s}"));
        source = std::error::Error::source(s);
    }

    match err {
        CliError::NotAuthenticated => {
            out.push_str("\n\nhint: Run `jsscli account add` then `jsscli auth login`.");
        }
        CliError::AccountNotFound(name) => {
            out.push_str(&format!(
                "\n\nhint: Run `jsscli account list` to see available accounts, \
                 or `jsscli account add {name}` to create one."
            ));
        }
        CliError::Jss(JssError::NotAuthenticated) => {
            out.push_str("\n\nhint: The session has expired. Run `jsscli auth login`.");
        }
        _ => {}
    }

    out
}

pub fn exit_code(err: &CliError) -> i32 {
    match err {
        CliError::Usage(_) => 2,
        CliError::NotAuthenticated => 3,
        CliError::Network(_) => 4,
        CliError::Api(_) => 5,
        _ => 1,
    }
}
