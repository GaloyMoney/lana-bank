//! Generate JSON schemas for public domain events.
//!
//! This binary generates JSON Schema for the LanaEvent enum, which encompasses
//! all public domain events in the Lana Bank system.
//!
//! Usage:
//!   cargo run --package public-events-schema --features json-schema

#[cfg(feature = "json-schema")]
fn main() {
    use schemars::schema_for;
    use std::fs;
    use std::path::Path;

    let schema = schema_for!(lana_events::LanaEvent);
    let json = serde_json::to_string_pretty(&schema).expect("Failed to serialize schema");

    // Output to stdout by default
    println!("{json}");

    // Also write to docs-site/schemas directory if it exists
    let schemas_dir = Path::new("docs-site/schemas");
    if schemas_dir.exists() || fs::create_dir_all(schemas_dir).is_ok() {
        let output_path = schemas_dir.join("lana_events.json");
        fs::write(&output_path, &json).expect("Failed to write schema file");
        eprintln!("Schema written to: {}", output_path.display());
    }
}

#[cfg(not(feature = "json-schema"))]
fn main() {
    eprintln!("Error: This binary requires the 'json-schema' feature.");
    eprintln!("Run with: cargo run --package public-events-schema --features json-schema");
    std::process::exit(1);
}
