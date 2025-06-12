use colored::*;
use core_access::{permission_set::PermissionSetEvent, role::RoleEvent, user::UserEvent};
use schemars::schema_for;
use serde_json::Value;
use std::fs;
use std::path::Path;

struct SchemaInfo {
    name: &'static str,
    filename: &'static str,
    generate_schema: fn() -> serde_json::Value,
}

pub fn update_schemas(schemas_out_dir: &str) -> anyhow::Result<()> {
    let schemas = vec![
        SchemaInfo {
            name: "UserEvent",
            filename: "user_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(UserEvent)).unwrap(),
        },
        SchemaInfo {
            name: "RoleEvent",
            filename: "role_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(RoleEvent)).unwrap(),
        },
        SchemaInfo {
            name: "PermissionSetEvent",
            filename: "permission_set_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(PermissionSetEvent)).unwrap(),
        },
    ];

    let schemas_dir = Path::new(schemas_out_dir);
    if !schemas_dir.exists() {
        fs::create_dir_all(schemas_dir)?;
    }

    let mut has_breaking_changes = false;

    for schema_info in schemas {
        let filepath = schemas_dir.join(schema_info.filename);
        let new_schema = (schema_info.generate_schema)();
        let new_schema_pretty = serde_json::to_string_pretty(&new_schema)?;

        if filepath.exists() {
            let existing_content = fs::read_to_string(&filepath)?;
            let existing_schema: Value = serde_json::from_str(&existing_content)?;

            if existing_schema != new_schema {
                println!("{} {}", "Schema changed:".yellow().bold(), schema_info.name);

                // Show diff
                show_diff(&existing_content, &new_schema_pretty);

                // Check for breaking changes
                if is_breaking_change(&existing_schema, &new_schema)? {
                    println!(
                        "{} Breaking change detected in {}",
                        "❌".red(),
                        schema_info.name.red().bold()
                    );
                    has_breaking_changes = true;
                } else {
                    println!(
                        "{} Non-breaking change in {}",
                        "✅".green(),
                        schema_info.name.green()
                    );
                }
            } else {
                println!("{} {} (no changes)", "✅".green(), schema_info.name);
            }
        } else {
            println!("{} Creating new schema: {}", "📝".blue(), schema_info.name);
        }

        // Write the new schema
        fs::write(&filepath, new_schema_pretty)?;
    }

    if has_breaking_changes {
        println!("\n{} Breaking changes detected!", "❌".red().bold());
        std::process::exit(1);
    } else {
        println!(
            "\n{} All schemas updated successfully!",
            "✅".green().bold()
        );
    }

    Ok(())
}

fn show_diff(old_content: &str, new_content: &str) {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    // Simple line-by-line diff
    let max_lines = old_lines.len().max(new_lines.len());

    for i in 0..max_lines {
        let old_line = old_lines.get(i).unwrap_or(&"");
        let new_line = new_lines.get(i).unwrap_or(&"");

        if old_line != new_line {
            if !old_line.is_empty() {
                println!("{} {}", "-".red(), old_line.red());
            }
            if !new_line.is_empty() {
                println!("{} {}", "+".green(), new_line.green());
            }
        }
    }
    println!();
}

fn is_breaking_change(old_schema: &Value, new_schema: &Value) -> anyhow::Result<bool> {
    // Basic breaking change detection
    // Check if required fields were removed or types changed

    if let (Some(old_props), Some(new_props)) =
        (old_schema.get("properties"), new_schema.get("properties"))
    {
        if let (Some(old_obj), Some(new_obj)) = (old_props.as_object(), new_props.as_object()) {
            // Check if any required properties were removed
            if let Some(old_required) = old_schema.get("required").and_then(|r| r.as_array()) {
                if let Some(new_required) = new_schema.get("required").and_then(|r| r.as_array()) {
                    for old_req in old_required {
                        if !new_required.contains(old_req) {
                            return Ok(true); // Required field removed
                        }
                    }
                }
            }

            // Check if property types changed in incompatible ways
            for (prop_name, old_prop) in old_obj {
                if let Some(new_prop) = new_obj.get(prop_name) {
                    if let (Some(old_type), Some(new_type)) = (
                        old_prop.get("type").and_then(|t| t.as_str()),
                        new_prop.get("type").and_then(|t| t.as_str()),
                    ) {
                        if old_type != new_type {
                            return Ok(true); // Type changed
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}