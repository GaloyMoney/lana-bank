use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use heck::ToTitleCase;
use serde_json::json;

fn main() {
    // Reference admin_server to ensure all domain crates are linked,
    // which registers their permission set entries via linkme.
    let _ = admin_server::graphql::schema(None);

    // 1. Permission labels
    let mut permissions = BTreeMap::new();
    for entry in permission_sets_macro::all_entries() {
        let label = entry.name.to_title_case();
        permissions.insert(
            entry.name,
            json!({
                "label": label,
                "description": entry.description,
            }),
        );
    }

    // 2. Transaction descriptions (parsed from CALA template source files)
    let descriptions = discover_transaction_descriptions();
    let mut desc_map = BTreeMap::new();
    for desc in &descriptions {
        desc_map.insert(desc.as_str(), desc.as_str());
    }

    // 3. Template codes → human-readable labels (e.g. "ACTIVATE_CREDIT_FACILITY" → "Activate Credit Facility")
    let codes = discover_template_codes();
    let mut code_map = BTreeMap::new();
    for code in &codes {
        let label = code.replace('_', " ").to_title_case();
        code_map.insert(code.as_str(), label);
    }

    // Write all sections into a single generated file
    write_json(
        "apps/admin-panel/messages/generated/en.json",
        json!({
            "Permissions": permissions,
            "TemplateCodes": code_map,
            "TransactionDescriptions": desc_map,
        }),
    );
}

/// Discover all `templates/` directories under `core/`.
fn discover_template_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    for entry in walkdir::WalkDir::new("core")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && entry.file_name() == "templates" {
            dirs.push(entry.path().to_path_buf());
        }
    }
    dirs
}

/// Parse CALA template files for `pub const ...: &str = "CODE"` to discover template codes.
/// Handles definitions that span multiple lines.
fn discover_template_codes() -> BTreeSet<String> {
    let mut codes = BTreeSet::new();

    for dir in discover_template_dirs() {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_some_and(|ext| ext == "rs") {
                let content =
                    std::fs::read_to_string(entry.path()).expect("failed to read template file");
                // Collapse whitespace so multi-line const defs become single-line
                let collapsed = content
                    .lines()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join(" ");
                for segment in collapsed.split("pub const ").skip(1) {
                    if !segment.contains(": &str =") {
                        continue;
                    }
                    if let Some(start) = segment.find('"')
                        && let Some(end) = segment[start + 1..].find('"')
                    {
                        let value = &segment[start + 1..start + 1 + end];
                        if value.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                            codes.insert(value.to_string());
                        }
                    }
                }
            }
        }
    }
    codes
}

/// Parse CALA template files for `.description("'...'")` patterns.
/// These are CEL string literals — the single quotes distinguish TX descriptions
/// from parameter descriptions (no quotes) and dynamic descriptions (format! calls).
fn discover_transaction_descriptions() -> BTreeSet<String> {
    let mut descriptions = BTreeSet::new();
    let pattern = ".description(\"'";

    for dir in discover_template_dirs() {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_some_and(|ext| ext == "rs") {
                let content =
                    std::fs::read_to_string(entry.path()).expect("failed to read template file");
                for line in content.lines() {
                    if let Some(start) = line.find(pattern) {
                        let after = &line[start + pattern.len()..];
                        if let Some(end) = after.find("'\"") {
                            descriptions.insert(after[..end].to_string());
                        }
                    }
                }
            }
        }
    }
    descriptions
}

fn write_json(path: &str, value: serde_json::Value) {
    let content = serde_json::to_string_pretty(&value).expect("failed to serialize JSON");
    std::fs::create_dir_all(Path::new(path).parent().expect("path has no parent"))
        .expect("failed to create directory");
    std::fs::write(path, format!("{content}\n")).expect("failed to write file");
}
