use crate::cli::Cli;
use crate::config::Settings;
use crate::error::CliResult;
use crate::output;
use crate::session::SessionRecord;

pub async fn set(_cli: &Cli, key: String, value: String) -> CliResult<()> {
    let mut settings = Settings::load()?;
    settings.set(&key, &value)?;
    settings.save()?;
    output::success(format!("{key} = {value}"));
    Ok(())
}

pub async fn get(_cli: &Cli, key: String) -> CliResult<()> {
    let settings = Settings::load()?;
    match settings.get(&key) {
        Some(v) => println!("{v}"),
        None => println!("(not set)"),
    }
    Ok(())
}

pub async fn list(_cli: &Cli) -> CliResult<()> {
    let settings = Settings::load()?;
    let pairs = settings.list_pairs();
    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(k, serde_json::Value::String(v));
    }
    let v = serde_json::Value::Object(map);
    crate::output::print_data(&v, crate::cli::OutputFormat::Table)?;
    Ok(())
}

pub async fn init(_cli: &Cli) -> CliResult<()> {
    Settings::ensure_dirs()?;
    let _settings = Settings::load()?;
    let _accounts = crate::config::AccountStore::load()?;
    let _ = SessionRecord::path();
    output::success("Initialized jsscli configuration");
    println!("  config:   {}", Settings::path()?.display());
    println!(
        "  accounts: {}",
        crate::config::AccountStore::path()?.display()
    );
    println!("  session:  {}", SessionRecord::path()?.display());
    Ok(())
}

pub async fn path_cmd(_cli: &Cli) -> CliResult<()> {
    println!("{}", Settings::path()?.display());
    Ok(())
}
