use anyhow::anyhow;
use colored::*;
use handlebars::Handlebars;
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

use super::SchemaInfo;

#[derive(serde::Serialize)]
struct RollupTableContext {
    entity_name: String,
    rollup_table_name: String,
    fields: Vec<FieldDefinition>,
}

#[derive(serde::Serialize)]
struct FieldDefinition {
    name: String,
    sql_type: String,
    nullable: bool,
}

pub fn generate_rollup_migrations(
    schemas: &[SchemaInfo],
    schemas_dir: &Path,
    migrations_out_dir: &str,
) -> anyhow::Result<()> {
    let migrations_dir = Path::new(migrations_out_dir);
    if !migrations_dir.exists() {
        fs::create_dir_all(migrations_dir)?;
    }

    // Read template from file
    let template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("templates")
        .join("rollup_table.sql.hbs");
    let template_content = fs::read_to_string(&template_path)?;

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("rollup_table", &template_content)?;

    for schema_info in schemas {
        // Read the schema to extract fields
        let schema_path = schemas_dir.join(schema_info.filename);
        let schema_content = fs::read_to_string(&schema_path)?;
        let schema: Value = serde_json::from_str(&schema_content)?;

        // Extract fields from the schema
        let fields = extract_fields_from_schema(&schema)?;

        // Generate table names from entity name
        // e.g., UserEvent -> core_user_events_rollup
        let entity_base = schema_info.name.replace("Event", "");
        let rollup_table_name = format!("core_{}_events_rollup", to_snake_case(&entity_base));

        let context = RollupTableContext {
            entity_name: schema_info.name.to_string(),
            rollup_table_name: rollup_table_name.clone(),
            fields,
        };

        let migration_content = handlebars.render("rollup_table", &context)?;

        // Generate timestamp for migration filename
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let migration_filename = format!("{}_{}.sql", timestamp, rollup_table_name);
        let migration_path = migrations_dir.join(migration_filename);

        fs::write(&migration_path, migration_content)?;
        println!(
            "{} Generated migration: {}",
            "✅".green(),
            migration_path.display()
        );
    }

    Ok(())
}

fn extract_fields_from_schema(schema: &Value) -> anyhow::Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();
    let mut all_properties = HashMap::new();

    // Get oneOf variants
    if let Some(Value::Array(one_of)) = schema.get("oneOf") {
        for variant in one_of {
            if let Some(Value::Object(properties)) = variant.get("properties") {
                for (prop_name, prop_schema) in properties {
                    if prop_name == "type" || prop_name == "id" {
                        continue; // Skip the discriminator field and id (handled as common field)
                    }
                    all_properties.insert(prop_name.clone(), prop_schema.clone());
                }
            }
        }
    }

    // Convert properties to field definitions
    for (name, prop_schema) in all_properties {
        let sql_type = json_schema_to_sql_type(&prop_schema)?;
        let nullable = true; // Since fields come from different oneOf variants, they should be nullable

        fields.push(FieldDefinition {
            name: to_snake_case(&name),
            sql_type,
            nullable,
        });
    }

    // Sort fields for consistent ordering
    fields.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(fields)
}

fn json_schema_to_sql_type(schema: &Value) -> anyhow::Result<String> {
    // Handle $ref
    if let Some(Value::String(ref_path)) = schema.get("$ref") {
        // For now, handle common refs
        if ref_path.contains("AuditInfo") {
            return Ok("JSONB".to_string());
        } else if ref_path.contains("AuditEntryId") {
            return Ok("BIGINT".to_string());
        }
    }

    // Handle direct types
    if let Some(Value::String(type_str)) = schema.get("type") {
        let sql_type = match type_str.as_str() {
            "string" => {
                if let Some(Value::String(format)) = schema.get("format") {
                    match format.as_str() {
                        "uuid" => "UUID",
                        "date-time" => "TIMESTAMPTZ",
                        _ => "TEXT",
                    }
                } else {
                    "TEXT"
                }
            }
            "integer" => {
                if let Some(Value::String(format)) = schema.get("format") {
                    match format.as_str() {
                        "int64" => "BIGINT",
                        _ => "INTEGER",
                    }
                } else {
                    "INTEGER"
                }
            }
            "number" => "NUMERIC",
            "boolean" => "BOOLEAN",
            "object" => "JSONB",
            "array" => "JSONB",
            _ => return Err(anyhow!("Unknown JSON schema type: {}", type_str)),
        };
        Ok(sql_type.to_string())
    } else {
        // Default to JSONB for complex types
        Ok("JSONB".to_string())
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 && !prev_was_upper {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
        prev_was_upper = ch.is_uppercase();
    }

    result
}