use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use heck::ToTitleCase;
use serde_json::json;

const TEMPLATE_DIRS: &[&str] = &[
    "core/deposit/src/ledger/templates",
    "core/credit/src/ledger/templates",
    "core/credit/src/collateral/ledger/templates",
    "core/credit/collection/src/ledger/templates",
];

fn main() {
    // Reference admin_server to ensure all domain crates are linked,
    // which registers their permission set entries via inventory.
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
    write_json(
        "apps/admin-panel/messages/permissions/en.json",
        json!({ "Permissions": permissions }),
    );

    // 2. Transaction descriptions (parsed from CALA template source files)
    let descriptions = discover_transaction_descriptions();
    let mut desc_map = BTreeMap::new();
    for desc in &descriptions {
        desc_map.insert(desc.as_str(), desc.as_str());
    }
    write_json(
        "apps/admin-panel/messages/transactions/en.json",
        json!({ "TransactionDescriptions": desc_map }),
    );
}

/// Parse CALA template files for `.description("'...'")` patterns.
/// These are CEL string literals — the single quotes distinguish TX descriptions
/// from parameter descriptions (no quotes) and dynamic descriptions (format! calls).
fn discover_transaction_descriptions() -> BTreeSet<String> {
    let mut descriptions = BTreeSet::new();
    let pattern = ".description(\"'";

    for dir in TEMPLATE_DIRS {
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
