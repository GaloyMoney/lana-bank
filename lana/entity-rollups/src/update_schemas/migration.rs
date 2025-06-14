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
    events_table_name: String,
    fields: Vec<FieldDefinition>,
}

#[derive(serde::Serialize)]
struct FieldDefinition {
    name: String,
    sql_type: String,
    nullable: bool,
    is_json_extract: bool,
    json_path: String,
    cast_type: Option<String>,
    revoke_events: Option<Vec<String>>,
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

    // Read template files
    let table_template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("templates")
        .join("rollup_table_only.sql.hbs");
    let table_template_content = fs::read_to_string(&table_template_path)?;

    let trigger_function_template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("templates")
        .join("rollup_trigger_function.sql.hbs");
    let trigger_function_template_content = fs::read_to_string(&trigger_function_template_path)?;

    let trigger_creation_template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("templates")
        .join("rollup_trigger_creation.sql.hbs");
    let trigger_creation_template_content = fs::read_to_string(&trigger_creation_template_path)?;

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("rollup_table_only", &table_template_content)?;
    handlebars.register_template_string("rollup_trigger_function", &trigger_function_template_content)?;
    handlebars.register_template_string("rollup_trigger_creation", &trigger_creation_template_content)?;

    for schema_info in schemas {
        // Read the schema to extract fields
        let schema_path = schemas_dir.join(schema_info.filename);
        let schema_content = fs::read_to_string(&schema_path)?;
        let schema: Value = serde_json::from_str(&schema_content)?;

        // Extract fields from the schema
        let fields = extract_fields_from_schema(&schema)?;

        // Generate table names from entity name
        // e.g., UserEvent -> core_user_events_rollup, core_user_events
        let entity_base = schema_info.name.replace("Event", "");
        let table_base = format!("{}_{}", schema_info.table_prefix, to_snake_case(&entity_base));
        let rollup_table_name = format!("{}_events_rollup", table_base);
        let events_table_name = format!("{}_events", table_base);

        let context = RollupTableContext {
            entity_name: schema_info.name.to_string(),
            rollup_table_name: rollup_table_name.clone(),
            events_table_name,
            fields,
        };

        // Render all template parts
        let table_content = handlebars.render("rollup_table_only", &context)?;
        let trigger_function_content = handlebars.render("rollup_trigger_function", &context)?;
        let trigger_creation_content = handlebars.render("rollup_trigger_creation", &context)?;
        // Combine all parts into one migration
        let migration_content = format!(
            "{}\n\n{}\n\n{}\n",
            table_content, trigger_function_content, trigger_creation_content
        );

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
    let mut field_revoke_events: HashMap<String, Vec<String>> = HashMap::new();

    // Get oneOf variants and analyze event types
    if let Some(Value::Array(one_of)) = schema.get("oneOf") {
        for variant in one_of {
            if let Some(Value::Object(properties)) = variant.get("properties") {
                // Get event type
                let event_type = if let Some(Value::Object(type_obj)) = properties.get("type") {
                    if let Some(Value::Array(enum_vals)) = type_obj.get("enum") {
                        if let Some(Value::String(type_name)) = enum_vals.get(0) {
                            Some(type_name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                for (prop_name, prop_schema) in properties {
                    if prop_name == "type" || prop_name == "id" || prop_name == "audit_info" {
                        continue; // Skip the discriminator field, id (handled as common field), and audit_info
                    }

                    // Track which fields this event type can modify
                    all_properties.insert(prop_name.clone(), prop_schema.clone());

                    // Special handling for revoke events - only certain field patterns are revoked
                    if let Some(ref event_type_name) = event_type {
                        if event_type_name.ends_with("_revoked")
                            || event_type_name.contains("revoke")
                        {
                            // Only the core field being revoked should be NULL'd, not auxiliary fields like audit_info
                            if should_field_be_revoked(prop_name, event_type_name) {
                                field_revoke_events
                                    .entry(prop_name.clone())
                                    .or_insert_with(Vec::new)
                                    .push(event_type_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Convert properties to field definitions
    for (name, prop_schema) in all_properties {
        let sql_type = json_schema_to_sql_type(&prop_schema)?;
        let nullable = true; // Since fields come from different oneOf variants, they should be nullable

        // Determine cast type for trigger function
        let cast_type = get_cast_type(&sql_type);

        // Get revoke events for this field
        let revoke_events = field_revoke_events.get(&name).cloned();

        fields.push(FieldDefinition {
            name: to_snake_case(&name),
            sql_type,
            nullable,
            is_json_extract: true,
            json_path: name.clone(),
            cast_type,
            revoke_events,
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
                        _ => "VARCHAR",
                    }
                } else {
                    "VARCHAR"
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

fn should_field_be_revoked(field_name: &str, event_type: &str) -> bool {
    // Ignore audit_info completely
    if field_name == "audit_info" {
        return false;
    }

    // If event type contains "revoked", all fields (except audit_info) are being revoked
    event_type.contains("revoked")
}

fn get_cast_type(sql_type: &str) -> Option<String> {
    match sql_type {
        "UUID" => Some("UUID".to_string()),
        "BIGINT" => Some("BIGINT".to_string()),
        "INTEGER" => Some("INTEGER".to_string()),
        "NUMERIC" => Some("NUMERIC".to_string()),
        "BOOLEAN" => Some("BOOLEAN".to_string()),
        "TIMESTAMPTZ" => Some("TIMESTAMPTZ".to_string()),
        _ => None, // TEXT and JSONB don't need casting from JSON strings
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
