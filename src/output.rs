use crate::cli::OutputFormat;
use crate::error::CliResult;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use serde_json::Value;

pub fn print_data(value: &Value, fmt: OutputFormat) -> CliResult<()> {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(value)?);
        }
        OutputFormat::Pretty => {
            if atty_stdout() {
                let colored =
                    colored_json::to_colored_json_auto(value).unwrap_or_else(|_| value.to_string());
                println!("{colored}");
            } else {
                println!("{}", serde_json::to_string_pretty(value)?);
            }
        }
        OutputFormat::Raw => match value {
            Value::String(s) => println!("{s}"),
            _ => println!("{}", serde_json::to_string(value)?),
        },
        OutputFormat::Table => {
            print_table(value)?;
        }
    }
    Ok(())
}

pub fn print_string(s: &str, fmt: OutputFormat) -> CliResult<()> {
    if let Ok(v) = serde_json::from_str::<Value>(s) {
        return print_data(&v, fmt);
    }
    println!("{s}");
    Ok(())
}

fn print_table(value: &Value) -> CliResult<()> {
    if let Value::Object(root) = value {
        if let Some(Value::Array(items)) = root.get("data") {
            print_array_as_table(items)?;
            return Ok(());
        }
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
        table.set_header(["Key", "Value"]);
        for (k, v) in root {
            table.add_row([k.clone(), stringify(v)]);
        }
        println!("{table}");
        return Ok(());
    }

    if let Value::Array(items) = value {
        print_array_as_table(items)?;
        return Ok(());
    }

    println!("{}", stringify(value));
    Ok(())
}

fn print_array_as_table(items: &[Value]) -> CliResult<()> {
    if items.is_empty() {
        println!("(tidak ada data)");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if let Some(Value::Object(first)) = items.first() {
        let headers: Vec<String> = first.keys().cloned().collect();
        table.set_header(
            headers
                .iter()
                .map(|h| Cell::new(h).fg(comfy_table::Color::Cyan)),
        );

        for item in items {
            if let Value::Object(obj) = item {
                let cells: Vec<String> = headers
                    .iter()
                    .map(|h| obj.get(h).map(stringify).unwrap_or_default())
                    .collect();
                table.add_row(cells);
            } else {
                table.add_row([stringify(item)]);
            }
        }
    } else {
        table.set_header([Cell::new("Value")]);
        for item in items {
            table.add_row([stringify(item)]);
        }
    }

    println!("{table}");
    Ok(())
}

fn stringify(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn atty_stdout() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

pub fn info(msg: impl AsRef<str>) {
    use colored::Colorize;
    eprintln!("{} {}", "i".cyan().bold(), msg.as_ref());
}

pub fn success(msg: impl AsRef<str>) {
    use colored::Colorize;
    eprintln!("{} {}", "✓".green().bold(), msg.as_ref());
}

pub fn warn(msg: impl AsRef<str>) {
    use colored::Colorize;
    eprintln!("{} {}", "!".yellow().bold(), msg.as_ref());
}
