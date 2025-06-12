use clap::{Args, Parser, Subcommand};
use colored::*;
use core_access::{permission_set::PermissionSetEvent, role::RoleEvent, user::UserEvent};
use entity_rollups::is_breaking_change;
use schemars::schema_for;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "entity-rollups")]
#[command(about = "A tool for managing entity rollup schemas")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    UpdateSchemas(UpdateSchemasArgs),
}

#[derive(Args)]
struct UpdateSchemasArgs {
    // No additional arguments needed for now
}

struct SchemaInfo {
    name: &'static str,
    filename: &'static str,
    generate_schema: fn() -> serde_json::Value,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::UpdateSchemas(_) => update_schemas(),
    }
}

fn update_schemas() -> anyhow::Result<()> {
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

    let schemas_dir = Path::new("schemas");
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
