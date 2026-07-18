use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "jsscli",
    version,
    about = "JSON-first CLI for the JSS / Frappe REST API",
    long_about = "jsscli manages authentication, accounts, configuration,\n\
                  and CRUD against Frappe/JSS doctypes, files, and reports.\n\
                  All output is JSON."
)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long, global = true, env = "JSS_PROFILE")]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum AuthModeKind {
    Session,
    Token,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Manual,
    Info,
    Changelog {
        #[arg(short = 'n', long, default_value_t = 50)]
        count: usize,
    },
    Auth {
        #[command(subcommand)]
        cmd: AuthCommand,
    },
    Account {
        #[command(subcommand)]
        cmd: AccountCommand,
    },
    Config {
        #[command(subcommand)]
        cmd: ConfigCommand,
    },
    Doctype {
        #[command(subcommand)]
        cmd: DoctypeCommand,
    },
    File {
        #[command(subcommand)]
        cmd: FileCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AuthCommand {
    Login,
    Logout,
    Status,
    Whoami,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AccountCommand {
    Add {
        name: String,
        #[arg(long)]
        url: Option<String>,
        #[arg(long, value_enum, default_value_t = AuthModeKind::Session)]
        mode: AuthModeKind,
        #[arg(long, env = "JSS_EMAIL")]
        email: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        api_secret: Option<String>,
        #[arg(long)]
        sitename: Option<String>,
        #[arg(long)]
        activate: bool,
    },
    List,
    Use {
        name: String,
    },
    Remove {
        name: String,
    },
    Show {
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommand {
    Set { key: String, value: String },
    Get { key: String },
    List,
    Init,
    Path,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DoctypeCommand {
    Get {
        doctype: String,
        name: String,
    },
    List {
        doctype: String,
        #[arg(long, value_delimiter = ',')]
        fields: Option<Vec<String>>,
        #[arg(long, value_name = "FIELD,OP,VALUE")]
        filter: Option<Vec<String>>,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long, default_value_t = 0)]
        start: u32,
        #[arg(long)]
        order_by: Option<String>,
    },
    Create {
        doctype: String,
        #[arg(default_value = "-")]
        file: String,
    },
    Update {
        doctype: String,
        name: String,
        #[arg(default_value = "-")]
        file: String,
    },
    Delete {
        doctype: String,
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum FileCommand {
    Upload {
        file: String,
        doctype: String,
        docname: String,
        fieldname: String,
    },
    Download {
        url: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short = 'y', long)]
        yes: bool,
    },
}
