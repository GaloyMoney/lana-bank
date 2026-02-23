use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};
use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(headers);
    for row in rows {
        table.add_row(row);
    }
    println!("{table}");
}

pub fn print_kv(pairs: &[(&str, &str)]) {
    let max_key = pairs.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in pairs {
        println!("{:>width$}: {}", key, value, width = max_key);
    }
}

/// Display a serde_json::Value as a string, stripping quotes from string values.
pub fn scalar(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Convert a CLI string argument into a serde_json::Value for use as a GraphQL
/// scalar input. Tries to parse as integer, then float, then falls back to string.
/// This handles scalars like UsdCents (u64), AnnualRatePct (decimal), etc.
pub fn sval(s: String) -> serde_json::Value {
    if let Ok(n) = s.parse::<u64>() {
        serde_json::Value::Number(n.into())
    } else if let Ok(f) = s.parse::<f64>() {
        serde_json::json!(f)
    } else {
        serde_json::Value::String(s)
    }
}
