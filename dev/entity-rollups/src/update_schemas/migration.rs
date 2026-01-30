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
    toggle_fields: Vec<FieldDefinition>,
    event_types: Vec<EventTypeInfo>,
    event_updates: Vec<EventUpdateInfo>,
}

#[derive(serde::Serialize, Clone, Debug)]
struct EventTypeInfo {
    name: String,
    fields: Vec<String>, // Field names that this event can modify
}

#[derive(serde::Serialize, Clone, Debug)]
struct EventUpdateInfo {
    name: String,
    field_updates: Vec<ComputedFieldAction>,
}

#[derive(serde::Serialize, Clone, Debug)]
struct ComputedFieldAction {
    name: String,
    sql_type: String,
    nullable: bool,
    is_json_extract: bool,
    json_path: String,
    cast_type: Option<String>,
    is_set_field: bool,
    set_item_field: Option<String>,
    is_jsonb_field: bool,
    element_cast_type: Option<String>,
    is_jsonb_array: bool,
    is_toggle_field: bool,
    // Computed action flags
    is_field_update: bool,
    is_field_removal: bool,
    is_set_add: bool,
    is_set_remove: bool,
    is_toggle_set: bool,
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
    toggle_fields: Vec<FieldDefinition>,
    event_types: Vec<EventTypeInfo>,
    event_updates: Vec<EventUpdateInfo>,
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
    element_cast_type: Option<String>,
    is_jsonb_array: bool,
    is_toggle_field: bool,
    toggle_events: Option<Vec<String>>,
}

fn compute_event_updates(
    fields: &[FieldDefinition],
    event_types: &[EventTypeInfo],
) -> Vec<EventUpdateInfo> {
    let mut event_updates = Vec::new();

    for event_type in event_types {
        let mut field_updates = Vec::new();

        for field in fields {
            let mut computed_field = ComputedFieldAction {
                name: field.name.clone(),
                sql_type: field.sql_type.clone(),
                nullable: field.nullable,
                is_json_extract: field.is_json_extract,
                json_path: field.json_path.clone(),
                cast_type: field.cast_type.clone(),
                is_set_field: field.is_set_field,
                set_item_field: field.set_item_field.clone(),
                is_jsonb_field: field.is_jsonb_field,
                element_cast_type: field.element_cast_type.clone(),
                is_jsonb_array: field.is_jsonb_array,
                is_toggle_field: field.is_toggle_field,
                is_field_update: false,
                is_field_removal: false,
                is_set_add: false,
                is_set_remove: false,
                is_toggle_set: false,
            };

            if field.is_set_field {
                // Handle array fields
                if let Some(ref add_events) = field.set_add_events {
                    // Convert PascalCase event names to snake_case for comparison
                    let snake_case_add_events: Vec<String> =
                        add_events.iter().map(|s| to_snake_case(s)).collect();
                    computed_field.is_set_add = snake_case_add_events.contains(&event_type.name);
                }
                if let Some(ref remove_events) = field.set_remove_events {
                    // Convert PascalCase event names to snake_case for comparison
                    let snake_case_remove_events: Vec<String> =
                        remove_events.iter().map(|s| to_snake_case(s)).collect();
                    computed_field.is_set_remove =
                        snake_case_remove_events.contains(&event_type.name);
                }
                // Also check if this collection field appears in the event (e.g., for initialization)
                if event_type.fields.contains(&field.json_path)
                    && !computed_field.is_set_add
                    && !computed_field.is_set_remove
                {
                    computed_field.is_field_update = true;
                }
            } else if field.is_toggle_field {
                // Handle toggle fields
                if let Some(ref toggle_events) = field.toggle_events {
                    // Convert PascalCase event names to snake_case for comparison
                    let snake_case_toggle_events: Vec<String> =
                        toggle_events.iter().map(|s| to_snake_case(s)).collect();
                    computed_field.is_toggle_set =
                        snake_case_toggle_events.contains(&event_type.name);
                }
            } else {
                // Handle regular fields
                if event_type.fields.contains(&field.json_path) {
                    // This event has this field
                    if let Some(ref revoke_events) = field.revoke_events {
                        computed_field.is_field_removal = revoke_events.contains(&event_type.name);
                        computed_field.is_field_update = !computed_field.is_field_removal;
                    } else {
                        computed_field.is_field_update = true;
                    }
                }
                // If the event doesn't have this field, all flags stay false (preserve current value)
            }

            // Only include fields that are actually being modified by this event
            if computed_field.is_field_update
                || computed_field.is_field_removal
                || computed_field.is_set_add
                || computed_field.is_set_remove
                || computed_field.is_toggle_set
            {
                field_updates.push(computed_field);
            }
        }

        event_updates.push(EventUpdateInfo {
            name: event_type.name.clone(),
            field_updates,
        });
    }

    event_updates
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

    // Embed template files at compile time
    let table_template_content = include_str!("../templates/rollup_table_only.sql.hbs");
    let trigger_function_template_content =
        include_str!("../templates/rollup_trigger_function.sql.hbs");
    let trigger_creation_template_content =
        include_str!("../templates/rollup_trigger_creation.sql.hbs");
    let alter_template_content = include_str!("../templates/rollup_table_alter.sql.hbs");

    // Embed fragment templates at compile time
    let field_update_fragment = include_str!("../templates/fragments/field_update.sql.hbs");
    let field_init_fragment = include_str!("../templates/fragments/field_init.sql.hbs");
    let field_init_only_fragment = include_str!("../templates/fragments/field_init.sql.hbs");
    let array_init_fragment = include_str!("../templates/fragments/array_init.sql.hbs");
    let array_update_fragment = include_str!("../templates/fragments/array_update.sql.hbs");
    let array_append_fragment = include_str!("../templates/fragments/array_append.sql.hbs");
    let array_removal_fragment = include_str!("../templates/fragments/array_removal.sql.hbs");
    let field_update_only_fragment = include_str!("../templates/fragments/field_update.sql.hbs");
    let field_update_basic_fragment =
        include_str!("../templates/fragments/field_update_basic.sql.hbs");
    let field_removal_fragment = include_str!("../templates/fragments/field_removal.sql.hbs");
    let field_preserve_fragment = include_str!("../templates/fragments/field_preserve.sql.hbs");
    let toggle_set_fragment = include_str!("../templates/fragments/toggle_set.sql.hbs");

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
    handlebars.register_helper(
        "contains",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let needle = h
                    .param(0)
                    .ok_or(handlebars::RenderErrorReason::MissingVariable(Some(
                        "contains: Missing first parameter".to_string(),
                    )))?;
                let haystack = h
                    .param(1)
                    .ok_or(handlebars::RenderErrorReason::MissingVariable(Some(
                        "contains: Missing second parameter".to_string(),
                    )))?;

                if let Some(array) = haystack.value().as_array() {
                    let contains = array.iter().any(|item| item == needle.value());
                    if contains {
                        out.write("true")?;
                    }
                }
                Ok(())
            },
        ),
    );
    handlebars.register_helper(
        "is_nested_path",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let path = h
                    .param(0)
                    .ok_or(handlebars::RenderErrorReason::MissingVariable(Some(
                        "is_nested_path: Missing path parameter".to_string(),
                    )))?;

                if let Some(path_str) = path.value().as_str() {
                    let is_nested = path_str.contains('.');
                    if is_nested {
                        out.write("true")?;
                    }
                }
                Ok(())
            },
        ),
    );
    handlebars.register_helper(
        "nested_json_extract",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let path = h
                    .param(0)
                    .ok_or(handlebars::RenderErrorReason::MissingVariable(Some(
                        "nested_json_extract: Missing path parameter".to_string(),
                    )))?;
                let is_json_op = h
                    .param(1)
                    .and_then(|p| p.value().as_bool())
                    .unwrap_or(false);

                if let Some(path_str) = path.value().as_str() {
                    let parts: Vec<&str> = path_str.split('.').collect();
                    if parts.len() > 1 {
                        let mut result = String::from("NEW.event");
                        for (i, part) in parts.iter().enumerate() {
                            if i == parts.len() - 1 && !is_json_op {
                                // Last part with ->> for text extraction
                                result.push_str(&format!(" ->> '{part}'"));
                            } else {
                                // Non-last parts or JSON extraction with ->
                                result.push_str(&format!(" -> '{part}'"));
                            }
                        }
                        out.write(&result)?;
                    } else {
                        // Single field
                        if is_json_op {
                            out.write(&format!("NEW.event -> '{path_str}'"))?;
                        } else {
                            out.write(&format!("NEW.event ->> '{path_str}'"))?;
                        }
                    }
                }
                Ok(())
            },
        ),
    );
    handlebars.register_helper(
        "path_exists_check",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let path = h
                    .param(0)
                    .ok_or(handlebars::RenderErrorReason::MissingVariable(Some(
                        "path_exists_check: Missing path parameter".to_string(),
                    )))?;

                if let Some(path_str) = path.value().as_str() {
                    let parts: Vec<&str> = path_str.split('.').collect();
                    if parts.len() > 1 {
                        let result = format!("NEW.event -> '{}' ? '{}'", parts[0], parts[1]);
                        out.write(&result)?;
                    } else {
                        let result = format!("NEW.event ? '{path_str}'");
                        out.write(&result)?;
                    }
                }
                Ok(())
            },
        ),
    );
    handlebars.register_template_string("rollup_table_only", table_template_content)?;
    handlebars
        .register_template_string("rollup_trigger_function", trigger_function_template_content)?;
    handlebars
        .register_template_string("rollup_trigger_creation", trigger_creation_template_content)?;
    handlebars.register_template_string("rollup_table_alter", alter_template_content)?;

    // Register fragment templates
    handlebars.register_template_string("field_update", field_update_fragment)?;
    handlebars.register_template_string("field_init", field_init_fragment)?;
    handlebars.register_template_string("field_init", field_init_only_fragment)?;
    handlebars.register_template_string("array_init", array_init_fragment)?;
    handlebars.register_template_string("array_update", array_update_fragment)?;
    handlebars.register_template_string("array_append", array_append_fragment)?;
    handlebars.register_template_string("array_removal", array_removal_fragment)?;
    handlebars.register_template_string("field_update", field_update_only_fragment)?;
    handlebars.register_template_string("field_update_basic", field_update_basic_fragment)?;
    handlebars.register_template_string("field_removal", field_removal_fragment)?;
    handlebars.register_template_string("field_preserve", field_preserve_fragment)?;
    handlebars.register_template_string("toggle_set", toggle_set_fragment)?;

    for schema_change in schema_changes {
        let schema_info = &schema_change.schema_info;

        // Extract fields and event types from the current schema using enhanced collections
        let (current_fields, event_types) = extract_fields_and_events_from_schema(
            &schema_change.current_schema,
            &schema_info.collections,
            &schema_info.delete_events,
            &schema_info.toggle_events,
        )?;

        // Separate fields into regular, collection, and toggle fields
        let (regular_fields, collection_fields, toggle_fields) = separate_fields(&current_fields);

        // Generate table names from entity name
        // e.g., UserEvent -> core_user_events_rollup, core_user_events
        let entity_base = schema_info.name.replace("Event", "");
        let table_base = format!(
            "{}_{}",
            schema_info.table_prefix,
            to_snake_case(&entity_base)
        );
        let rollup_table_name = format!("{table_base}_events_rollup");
        let events_table_name = format!("{table_base}_events");

        // Check if we have a previous schema to compare with
        if let Some(ref previous_schema) = schema_change.previous_schema {
            let (previous_fields, _) = extract_fields_and_events_from_schema(
                previous_schema,
                &schema_info.collections,
                &schema_info.delete_events,
                &schema_info.toggle_events,
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

            let event_updates = compute_event_updates(&current_fields, &event_types);
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
                toggle_fields: toggle_fields.clone(),
                event_types: event_types.clone(),
                event_updates: event_updates.clone(),
            };
            let trigger_context = RollupTableContext {
                entity_name: schema_info.name.to_string(),
                rollup_table_name: rollup_table_name.clone(),
                events_table_name,
                fields: current_fields,
                regular_fields: regular_fields.clone(),
                collection_fields: collection_fields.clone(),
                toggle_fields: toggle_fields.clone(),
                event_types: event_types.clone(),
                event_updates,
            };

            // Render templates
            let table_structure_content =
                handlebars.render("rollup_table_only", &trigger_context)?;
            let alter_content = handlebars.render("rollup_table_alter", &alter_context)?;
            let trigger_function_content =
                handlebars.render("rollup_trigger_function", &trigger_context)?;

            // Create current table structure comment
            let table_structure_comment = format!(
                "-- Current table structure after migration:\n/*\n{table_structure_content}\n*/\n"
            );

            // Combine templates
            let migration_content = format!(
                "{table_structure_comment}\n{alter_content}\n\n{trigger_function_content}\n"
            );

            // Generate timestamp for migration filename
            let timestamp = (base_timestamp + chrono::Duration::seconds(migration_counter))
                .format("%Y%m%d%H%M%S")
                .to_string();
            migration_counter += 1;
            let migration_filename = format!("{timestamp}_update_{rollup_table_name}.sql");
            let migration_path = migrations_dir.join(migration_filename);

            fs::write(&migration_path, migration_content)?;
            println!(
                "{} Generated update migration: {}",
                "✅".green(),
                migration_path.display()
            );
        } else {
            // Initial table creation
            let event_updates = compute_event_updates(&current_fields, &event_types);
            let context = RollupTableContext {
                entity_name: schema_info.name.to_string(),
                rollup_table_name: rollup_table_name.clone(),
                events_table_name,
                fields: current_fields,
                regular_fields,
                collection_fields,
                toggle_fields,
                event_types,
                event_updates,
            };

            // Render all template parts
            let table_content = handlebars.render("rollup_table_only", &context)?;
            let trigger_function_content =
                handlebars.render("rollup_trigger_function", &context)?;
            let trigger_creation_content =
                handlebars.render("rollup_trigger_creation", &context)?;

            // Combine all parts into one migration
            let migration_content = format!(
                "{table_content}\n\n{trigger_function_content}\n\n{trigger_creation_content}\n"
            );

            // Generate timestamp for migration filename
            let timestamp = (base_timestamp + chrono::Duration::seconds(migration_counter))
                .format("%Y%m%d%H%M%S")
                .to_string();
            migration_counter += 1;
            let migration_filename = format!("{timestamp}_create_{rollup_table_name}.sql");
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

fn extract_fields_and_events_from_schema(
    schema: &Value,
    collection_rollups: &[super::CollectionRollup],
    delete_events: &[&str],
    toggle_events: &[&str],
) -> anyhow::Result<(Vec<FieldDefinition>, Vec<EventTypeInfo>)> {
    let mut fields = Vec::new();
    let mut all_properties: Vec<(String, Value)> = Vec::new();
    let mut field_revoke_events: HashMap<String, Vec<String>> = HashMap::new();
    let mut event_types = Vec::new();

    // Track set field relationships
    struct SetFieldInfo {
        item_field_path: String,
        add_events: Vec<String>,
        remove_events: Vec<String>,
    }

    // Build set field info from collection rollups
    let mut set_field_info: HashMap<String, SetFieldInfo> = HashMap::new();
    for rollup in collection_rollups {
        set_field_info.insert(
            rollup.column_name.to_string(),
            SetFieldInfo {
                item_field_path: rollup.values.to_string(),
                add_events: rollup.add_events.clone(),
                remove_events: rollup.remove_events.clone(),
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
                        if let Some(Value::String(type_name)) = enum_vals.first() {
                            Some(type_name.clone())
                        } else {
                            None
                        }
                    } else if let Some(Value::String(const_val)) = type_obj.get("const") {
                        Some(const_val.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let mut event_field_names = Vec::new();
                // Sort properties for deterministic iteration
                let mut sorted_properties: Vec<(String, Value)> = properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                sorted_properties.sort_by(|a, b| a.0.cmp(&b.0));

                for (prop_name, prop_schema) in &sorted_properties {
                    if prop_name == "type" || prop_name == "id" {
                        continue; // Skip the discriminator field, id (handled as common field)
                    }

                    // Track which fields this event type can modify
                    // Only add if not already present to preserve first occurrence order
                    if !all_properties.iter().any(|(name, _)| name == prop_name) {
                        all_properties.push((prop_name.clone(), prop_schema.clone()));
                    }
                    event_field_names.push(to_snake_case(prop_name));

                    if let Some(ref event_type_name) = event_type {
                        // Check if this event type is in the delete_events list
                        // Convert delete_events from PascalCase to snake_case for comparison
                        let snake_case_delete_events: Vec<String> =
                            delete_events.iter().map(|&s| to_snake_case(s)).collect();

                        if snake_case_delete_events.contains(event_type_name) {
                            field_revoke_events
                                .entry(prop_name.clone())
                                .or_default()
                                .push(event_type_name.clone());
                        }
                    }
                }

                // Add event type info
                if let Some(event_type_name) = event_type {
                    // Sort event field names for deterministic output
                    event_field_names.sort();
                    event_types.push(EventTypeInfo {
                        name: event_type_name,
                        fields: event_field_names,
                    });
                }
            }
        }
    }

    // Add array fields from collection rollups that aren't already tracked
    // Sort the set field names for deterministic iteration
    let mut sorted_set_field_names: Vec<String> = set_field_info.keys().cloned().collect();
    sorted_set_field_names.sort();

    for set_field_name in &sorted_set_field_names {
        if !all_properties
            .iter()
            .any(|(name, _)| name == set_field_name)
        {
            // Create a synthetic UUID array schema for the set field
            let array_schema = serde_json::json!({
                "type": "array",
                "items": {
                    "type": "string",
                    "format": "uuid"
                }
            });
            all_properties.push((set_field_name.clone(), array_schema));
        }
    }

    // Convert properties to field definitions
    for (name, prop_schema) in &all_properties {
        let mut sql_type = json_schema_to_sql_type_with_definitions(prop_schema, Some(schema))?;
        let nullable = true; // Since fields come from different oneOf variants, they should be nullable

        // Skip the individual ID fields if they're part of a set
        if set_field_info
            .values()
            .any(|info| !info.item_field_path.contains('.') && info.item_field_path == *name)
        {
            continue;
        }

        // Check if this is a set field
        let is_set_field = set_field_info.contains_key(name);
        let (set_add_events, set_remove_events, set_item_field, element_cast_type, is_jsonb_array) =
            if let Some(info) = set_field_info.get(name) {
                // Determine array type based on the field path
                let item_type = if !info.item_field_path.contains('.') {
                    // Simple field path - look for it in all_properties
                    if let Some((_, item_schema)) = all_properties
                        .iter()
                        .find(|(name, _)| name == &info.item_field_path)
                    {
                        json_schema_to_sql_type_with_definitions(item_schema, Some(schema))
                            .unwrap_or_else(|_| "VARCHAR".to_string())
                    } else {
                        "VARCHAR".to_string()
                    }
                } else {
                    // Nested field path - lookup the definition and find the correct type
                    lookup_nested_field_type(schema, &info.item_field_path)
                        .unwrap_or_else(|_| "VARCHAR".to_string())
                };
                // If the item type is JSONB, store the entire array as JSONB
                let is_jsonb_array = item_type == "JSONB";
                sql_type = if is_jsonb_array {
                    "JSONB".to_string()
                } else {
                    format!("{item_type}[]")
                };
                // Calculate element cast type for arrays
                let element_cast_type = get_cast_type(&item_type);
                (
                    Some(info.add_events.clone()),
                    Some(info.remove_events.clone()),
                    Some(info.item_field_path.clone()),
                    element_cast_type,
                    is_jsonb_array,
                )
            } else {
                (None, None, None, None, false)
            };

        // Determine cast type for trigger function
        let cast_type = get_cast_type(&sql_type);

        // Get revoke events for this field
        let revoke_events = field_revoke_events.get(name).cloned();

        // Determine if this field should use JSONB extraction (-> operator vs ->> operator)
        let is_jsonb_field = sql_type == "JSONB";

        fields.push(FieldDefinition {
            name: to_snake_case(name),
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
            element_cast_type,
            is_jsonb_array,
            is_toggle_field: false,
            toggle_events: None,
        });
    }

    // Add toggle fields for toggle events
    // Sort toggle events for deterministic processing
    let mut sorted_toggle_events: Vec<&str> = toggle_events.to_vec();
    sorted_toggle_events.sort();

    for toggle_event in &sorted_toggle_events {
        let toggle_field_name = format!("is_{}", to_snake_case(toggle_event));

        // Check if field already exists
        if !fields.iter().any(|f| f.name == toggle_field_name) {
            fields.push(FieldDefinition {
                name: toggle_field_name.clone(),
                sql_type: "BOOLEAN".to_string(),
                nullable: false,       // toggle fields default to false, not null
                is_json_extract: true, // toggle fields can extract from JSON with COALESCE
                json_path: toggle_field_name.clone(),
                cast_type: None,
                revoke_events: None,
                is_set_field: false,
                set_add_events: None,
                set_remove_events: None,
                set_item_field: None,
                is_jsonb_field: false,
                element_cast_type: None,
                is_jsonb_array: false,
                is_toggle_field: true,
                toggle_events: Some(vec![toggle_event.to_string()]),
            });
        }
    }

    // Add collection events to event types
    for rollup in collection_rollups {
        for add_event in &rollup.add_events {
            let event_name = to_snake_case(add_event);
            if !event_types.iter().any(|et| et.name == event_name) {
                event_types.push(EventTypeInfo {
                    name: event_name,
                    fields: vec![rollup.column_name.to_string()],
                });
            }
        }
        for remove_event in &rollup.remove_events {
            let event_name = to_snake_case(remove_event);
            if !event_types.iter().any(|et| et.name == event_name) {
                event_types.push(EventTypeInfo {
                    name: event_name,
                    fields: vec![rollup.column_name.to_string()],
                });
            }
        }
    }

    // Sort fields for deterministic output
    fields.sort_by(|a, b| a.name.cmp(&b.name));

    // Keep event types in schema order (don't sort)

    Ok((fields, event_types))
}

fn is_primitive_wrapper(schema: &Value) -> bool {
    // A primitive wrapper has a "type" field that is a primitive type
    // and optionally format, minimum, maximum, enum, or other constraint fields
    if let Some(obj) = schema.as_object() {
        // Must have a type field
        if let Some(type_value) = obj.get("type") {
            // Check if it's a primitive type
            let is_primitive_type = match type_value {
                Value::String(s) => {
                    matches!(s.as_str(), "string" | "integer" | "number" | "boolean")
                }
                Value::Array(arr) => {
                    // For nullable types like ["string", "null"]
                    arr.iter().any(|v| {
                        if let Value::String(s) = v {
                            matches!(s.as_str(), "string" | "integer" | "number" | "boolean")
                        } else {
                            false
                        }
                    })
                }
                _ => false,
            };

            if !is_primitive_type {
                return false;
            }

            // Check that it doesn't have properties (which would make it an object)
            if obj.contains_key("properties") {
                return false;
            }

            // Check that it doesn't have complex composition fields
            if obj.contains_key("oneOf") || obj.contains_key("anyOf") || obj.contains_key("allOf") {
                return false;
            }

            // If it's an array type, it's not a primitive wrapper
            if let Value::String(s) = type_value
                && (s == "array" || s == "object")
            {
                return false;
            }

            return true;
        }
    }
    false
}

fn json_schema_to_sql_type_with_definitions(
    schema: &Value,
    definitions: Option<&Value>,
) -> anyhow::Result<String> {
    // Handle anyOf
    if let Some(obj) = schema.as_object() {
        if let Some(Value::Array(any_of_array)) = obj.get("anyOf") {
            // For simplicity, if anyOf includes null, treat as nullable of the other type
            for any_of_entry in any_of_array {
                if let Value::Object(entry_obj) = any_of_entry {
                    if let Some(Value::String(type_str)) = entry_obj.get("type") {
                        if type_str != "null" {
                            // Recursively determine the type of the non-null entry
                            return json_schema_to_sql_type_with_definitions(
                                any_of_entry,
                                definitions,
                            );
                        }
                    } else if let Some(Value::String(_ref_path)) = entry_obj.get("$ref") {
                        // Recursively determine the type of the non-null entry
                        return json_schema_to_sql_type_with_definitions(any_of_entry, definitions);
                    }
                }
            }
        }
    }

    // Handle $ref
    if let Some(Value::String(ref_path)) = schema.get("$ref") {
        // For now, handle common refs
        if ref_path.contains("AuditInfo") {
            return Ok("JSONB".to_string());
        } else if ref_path.contains("AuditEntryId") {
            return Ok("BIGINT".to_string());
        } else if ref_path.contains("PriceOfOneBTC") {
            return Ok("BIGINT".to_string());
        } else if ref_path.contains("EffectiveDate") {
            return Ok("TIMESTAMPTZ".to_string());
        }

        // Try to resolve other $refs if definitions are available
        if let Some(defs) = definitions {
            // Check both #/definitions/ and #/$defs/ patterns
            let def_name = ref_path
                .strip_prefix("#/definitions/")
                .or_else(|| ref_path.strip_prefix("#/$defs/"));

            if let Some(def_name) = def_name {
                // Look for the definition in both "definitions" and "$defs"
                let definition = defs
                    .get("definitions")
                    .and_then(|d| d.get(def_name))
                    .or_else(|| defs.get("$defs").and_then(|d| d.get(def_name)));

                if let Some(definition) = definition {
                    // Check if this is a primitive wrapper (has only type and optionally format/minimum/maximum)
                    if is_primitive_wrapper(definition) {
                        return json_schema_to_sql_type_with_definitions(definition, definitions);
                    } else {
                        // Complex type, return JSONB
                        return Ok("JSONB".to_string());
                    }
                }
            }
        }
    }

    // Handle direct types
    if let Some(type_value) = schema.get("type") {
        // Handle array types (nullable fields)
        if let Value::Array(type_array) = type_value {
            // For nullable types, find the non-null type
            for type_item in type_array {
                if let Value::String(type_str) = type_item
                    && type_str != "null"
                {
                    let sql_type = match type_str.as_str() {
                        "string" => {
                            // Check if this is an enum first
                            if schema.get("enum").is_some() {
                                "VARCHAR"
                            } else if let Some(Value::String(format)) = schema.get("format") {
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
                                    "int64" | "uint64" => "BIGINT",
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
                    return Ok(sql_type.to_string());
                }
            }
        } else if let Value::String(type_str) = type_value {
            let sql_type = match type_str.as_str() {
                "string" => {
                    // Check if this is an enum first
                    if schema.get("enum").is_some() {
                        "VARCHAR"
                    } else if let Some(Value::String(format)) = schema.get("format") {
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
                            "int64" | "uint64" => "BIGINT",
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
            return Ok(sql_type.to_string());
        }
    }

    // Default to JSONB for complex types
    Ok("JSONB".to_string())
}

fn lookup_nested_field_type(schema: &Value, field_path: &str) -> anyhow::Result<String> {
    let path_parts: Vec<&str> = field_path.split('.').collect();

    if path_parts.len() == 1 {
        // Simple field - should be handled elsewhere
        return Ok("VARCHAR".to_string());
    }

    // Look for the parent field in oneOf entries to find its type
    let parent_field = &path_parts[0];
    let nested_field = &path_parts[1];

    // Search through oneOf variants to find the parent field
    if let Some(Value::Array(one_of)) = schema.get("oneOf") {
        for variant in one_of {
            if let Some(Value::Object(properties)) = variant.get("properties")
                && let Some(parent_field_schema) = properties.get(*parent_field)
                // Found the parent field, now look up its type in definitions
                && let Some(Value::String(ref_path)) = parent_field_schema.get("$ref")
                && let Some(type_name) = ref_path.strip_prefix("#/definitions/")
                && let Some(Value::Object(definitions)) = schema.get("definitions")
                && let Some(type_def) = definitions.get(type_name)
                && let Some(Value::Object(type_properties)) =
                    type_def.get("properties")
                && let Some(nested_field_schema) =
                    type_properties.get(*nested_field)
            {
                return json_schema_to_sql_type_with_definitions(nested_field_schema, Some(schema));
            }
        }
    }

    // Fallback: if we can't find it, try to infer from common field names
    match path_parts.last() {
        Some(&"audit_entry_id") => Ok("BIGINT".to_string()),
        Some(field) if field.ends_with("_id") => Ok("UUID".to_string()),
        _ => Ok("VARCHAR".to_string()),
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

fn separate_fields(
    fields: &[FieldDefinition],
) -> (
    Vec<FieldDefinition>,
    Vec<FieldDefinition>,
    Vec<FieldDefinition>,
) {
    let mut regular_fields = Vec::new();
    let mut collection_fields = Vec::new();
    let mut toggle_fields = Vec::new();

    for field in fields {
        if field.is_set_field {
            collection_fields.push(field.clone());
        } else if field.is_toggle_field {
            toggle_fields.push(field.clone());
        } else {
            regular_fields.push(field.clone());
        }
    }

    (regular_fields, collection_fields, toggle_fields)
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
