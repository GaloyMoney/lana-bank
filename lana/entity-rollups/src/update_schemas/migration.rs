use anyhow::anyhow;
use colored::*;
use handlebars::Handlebars;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use super::SchemaChangeInfo;

#[derive(serde::Serialize)]
struct RollupTableContext {
    entity_name: String,
    rollup_table_name: String,
    events_table_name: String,
    fields: Vec<FieldDefinition>,
    regular_fields: Vec<FieldDefinition>,
    collection_fields: Vec<FieldDefinition>,
}

#[derive(serde::Serialize)]
struct RollupUpdateContext {
    entity_name: String,
    table_name: String,
    rollup_table_name: String,
    events_table_name: String,
    fields: Vec<FieldDefinition>,
    all_fields: Vec<FieldDefinition>,
    new_fields: Vec<FieldDefinition>,
    removed_fields: Vec<FieldDefinition>,
    regular_fields: Vec<FieldDefinition>,
    collection_fields: Vec<FieldDefinition>,
}

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
struct FieldDefinition {
    name: String,
    sql_type: String,
    nullable: bool,
    is_json_extract: bool,
    json_path: String,
    cast_type: Option<String>,
    revoke_events: Option<Vec<String>>,
    is_set_field: bool,
    set_add_events: Option<Vec<String>>,
    set_remove_events: Option<Vec<String>>,
    set_item_field: Option<String>,
    is_jsonb_field: bool,
}

pub fn generate_rollup_migrations(
    schema_changes: &[SchemaChangeInfo],
    migrations_out_dir: &str,
) -> anyhow::Result<()> {
    // Base timestamp for consistent ordering
    let base_timestamp = chrono::Utc::now();
    let mut migration_counter = 0;
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

    let alter_template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("templates")
        .join("rollup_table_alter.sql.hbs");
    let alter_template_content = fs::read_to_string(&alter_template_path)?;

    let mut handlebars = Handlebars::new();
    handlebars.register_helper(
        "eq",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let param1 = h
                    .param(0)
                    .ok_or(handlebars::RenderErrorReason::MissingVariable(Some(
                        "eq: Missing first parameter".to_string(),
                    )))?;
                let param2 = h
                    .param(1)
                    .ok_or(handlebars::RenderErrorReason::MissingVariable(Some(
                        "eq: Missing second parameter".to_string(),
                    )))?;

                let equals = param1.value() == param2.value();
                if equals {
                    out.write("true")?;
                }
                Ok(())
            },
        ),
    );
    handlebars.register_template_string("rollup_table_only", &table_template_content)?;
    handlebars.register_template_string(
        "rollup_trigger_function",
        &trigger_function_template_content,
    )?;
    handlebars.register_template_string(
        "rollup_trigger_creation",
        &trigger_creation_template_content,
    )?;
    handlebars.register_template_string("rollup_table_alter", &alter_template_content)?;

    for schema_change in schema_changes {
        let schema_info = &schema_change.schema_info;

        // Extract fields from the current schema
        let current_fields = extract_fields_from_schema(
            &schema_change.current_schema,
            &schema_info.collections,
            &schema_info.delete_events,
        )?;
        
        // Separate fields into regular and collection fields
        let (regular_fields, collection_fields) = separate_fields(&current_fields);

        // Generate table names from entity name
        // e.g., UserEvent -> core_user_events_rollup, core_user_events
        let entity_base = schema_info.name.replace("Event", "");
        let table_base = format!(
            "{}_{}",
            schema_info.table_prefix,
            to_snake_case(&entity_base)
        );
        let rollup_table_name = format!("{}_events_rollup", table_base);
        let events_table_name = format!("{}_events", table_base);

        // Check if we have a previous schema to compare with
        if let Some(ref previous_schema) = schema_change.previous_schema {
            let previous_fields = extract_fields_from_schema(
                previous_schema,
                &schema_info.collections,
                &schema_info.delete_events,
            )?;

            // Compare fields
            let (new_fields, removed_fields) = compare_fields(&previous_fields, &current_fields);

            if new_fields.is_empty() && removed_fields.is_empty() {
                println!(
                    "{} No changes in {}, skipping migration",
                    "ℹ️".blue(),
                    schema_info.name
                );
                continue;
            }

            let alter_context = RollupUpdateContext {
                entity_name: schema_info.name.to_string(),
                table_name: table_base.clone(),
                rollup_table_name: rollup_table_name.clone(),
                events_table_name: events_table_name.clone(),
                fields: current_fields.clone(),
                all_fields: current_fields.clone(),
                new_fields,
                removed_fields,
                regular_fields: regular_fields.clone(),
                collection_fields: collection_fields.clone(),
            };

            let trigger_context = RollupTableContext {
                entity_name: schema_info.name.to_string(),
                rollup_table_name: rollup_table_name.clone(),
                events_table_name,
                fields: current_fields,
                regular_fields: regular_fields.clone(),
                collection_fields: collection_fields.clone(),
            };

            // Render templates
            let table_structure_content =
                handlebars.render("rollup_table_only", &trigger_context)?;
            let alter_content = handlebars.render("rollup_table_alter", &alter_context)?;
            let trigger_function_content =
                handlebars.render("rollup_trigger_function", &trigger_context)?;

            // Create current table structure comment
            let table_structure_comment = format!(
                "-- Current table structure after migration:\n/*\n{}\n*/\n",
                table_structure_content
            );

            // Combine templates
            let migration_content = format!(
                "{}\n{}\n\n{}\n",
                table_structure_comment, alter_content, trigger_function_content
            );

            // Generate timestamp for migration filename
            let timestamp = (base_timestamp + chrono::Duration::seconds(migration_counter))
                .format("%Y%m%d%H%M%S")
                .to_string();
            migration_counter += 1;
            let migration_filename = format!("{}_update_{}.sql", timestamp, rollup_table_name);
            let migration_path = migrations_dir.join(migration_filename);

            fs::write(&migration_path, migration_content)?;
            println!(
                "{} Generated update migration: {}",
                "✅".green(),
                migration_path.display()
            );
        } else {
            // Initial table creation
            let context = RollupTableContext {
                entity_name: schema_info.name.to_string(),
                rollup_table_name: rollup_table_name.clone(),
                events_table_name,
                fields: current_fields,
                regular_fields,
                collection_fields,
            };

            // Render all template parts
            let table_content = handlebars.render("rollup_table_only", &context)?;
            let trigger_function_content =
                handlebars.render("rollup_trigger_function", &context)?;
            let trigger_creation_content =
                handlebars.render("rollup_trigger_creation", &context)?;

            // Combine all parts into one migration
            let migration_content = format!(
                "{}\n\n{}\n\n{}\n",
                table_content, trigger_function_content, trigger_creation_content
            );

            // Generate timestamp for migration filename
            let timestamp = (base_timestamp + chrono::Duration::seconds(migration_counter))
                .format("%Y%m%d%H%M%S")
                .to_string();
            migration_counter += 1;
            let migration_filename = format!("{}_create_{}.sql", timestamp, rollup_table_name);
            let migration_path = migrations_dir.join(migration_filename);

            fs::write(&migration_path, migration_content)?;
            println!(
                "{} Generated create migration: {}",
                "✅".green(),
                migration_path.display()
            );
        }
    }

    Ok(())
}

fn extract_fields_from_schema(
    schema: &Value,
    collection_rollups: &[super::CollectionRollup],
    delete_events: &[&str],
) -> anyhow::Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();
    let mut all_properties = HashMap::new();
    let mut field_revoke_events: HashMap<String, Vec<String>> = HashMap::new();

    // Track set field relationships
    struct SetFieldInfo {
        item_field_name: String,
        add_events: Vec<String>,
        remove_events: Vec<String>,
    }

    // Build set field info from collection rollups
    let mut set_field_info: HashMap<String, SetFieldInfo> = HashMap::new();
    for rollup in collection_rollups {
        set_field_info.insert(
            rollup.column_name.to_string(),
            SetFieldInfo {
                item_field_name: rollup.values.to_string(),
                add_events: rollup.add_events.iter().map(|s| s.to_string()).collect(),
                remove_events: rollup.remove_events.iter().map(|s| s.to_string()).collect(),
            },
        );
    }

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

                    if let Some(ref event_type_name) = event_type {
                        // Check if this event type is in the delete_events list
                        // Convert delete_events from PascalCase to snake_case for comparison
                        let snake_case_delete_events: Vec<String> = delete_events
                            .iter()
                            .map(|&s| to_snake_case(s))
                            .collect();
                        
                        if snake_case_delete_events.contains(&event_type_name) {
                            // Only add fields that aren't audit_info to the revoke list
                            if prop_name != "audit_info" {
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

    // Add array fields from collection rollups that aren't already tracked
    for (set_field_name, _) in &set_field_info {
        if !all_properties.contains_key(set_field_name) {
            // Create a synthetic UUID array schema for the set field
            let array_schema = serde_json::json!({
                "type": "array",
                "items": {
                    "type": "string",
                    "format": "uuid"
                }
            });
            all_properties.insert(set_field_name.clone(), array_schema);
        }
    }

    // Convert properties to field definitions
    for (name, prop_schema) in &all_properties {
        let mut sql_type = json_schema_to_sql_type(&prop_schema)?;
        let nullable = true; // Since fields come from different oneOf variants, they should be nullable

        // Skip the individual ID fields if they're part of a set
        if set_field_info
            .values()
            .any(|info| info.item_field_name == *name)
        {
            continue;
        }

        // Check if this is a set field
        let is_set_field = set_field_info.contains_key(name);
        let (set_add_events, set_remove_events, set_item_field) =
            if let Some(info) = set_field_info.get(name) {
                // Determine array type based on the individual field type
                let item_type = if let Some(item_schema) = all_properties.get(&info.item_field_name) {
                    json_schema_to_sql_type(item_schema).unwrap_or_else(|_| "VARCHAR".to_string())
                } else {
                    "VARCHAR".to_string()
                };
                sql_type = format!("{}[]", item_type);
                (
                    Some(info.add_events.clone()),
                    Some(info.remove_events.clone()),
                    Some(info.item_field_name.clone()),
                )
            } else {
                (None, None, None)
            };

        // Determine cast type for trigger function
        let cast_type = get_cast_type(&sql_type);

        // Get revoke events for this field
        let revoke_events = field_revoke_events.get(name).cloned();

        // Determine if this field should use JSONB extraction (-> operator vs ->> operator)
        let is_jsonb_field = sql_type == "JSONB";

        fields.push(FieldDefinition {
            name: to_snake_case(&name),
            sql_type,
            nullable,
            is_json_extract: true,
            json_path: name.clone(),
            cast_type,
            revoke_events,
            is_set_field,
            set_add_events,
            set_remove_events,
            set_item_field,
            is_jsonb_field,
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

fn compare_fields(
    previous: &[FieldDefinition],
    current: &[FieldDefinition],
) -> (Vec<FieldDefinition>, Vec<FieldDefinition>) {
    let previous_names: HashSet<String> = previous.iter().map(|f| f.name.clone()).collect();
    let current_names: HashSet<String> = current.iter().map(|f| f.name.clone()).collect();

    let new_fields: Vec<FieldDefinition> = current
        .iter()
        .filter(|f| !previous_names.contains(&f.name))
        .cloned()
        .collect();

    let removed_fields: Vec<FieldDefinition> = previous
        .iter()
        .filter(|f| !current_names.contains(&f.name))
        .cloned()
        .collect();

    (new_fields, removed_fields)
}

fn separate_fields(fields: &[FieldDefinition]) -> (Vec<FieldDefinition>, Vec<FieldDefinition>) {
    let mut regular_fields = Vec::new();
    let mut collection_fields = Vec::new();
    
    for field in fields {
        if field.is_set_field {
            collection_fields.push(field.clone());
        } else {
            regular_fields.push(field.clone());
        }
    }
    
    (regular_fields, collection_fields)
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
