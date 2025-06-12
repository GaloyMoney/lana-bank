use std::env;
use std::fs;
use std::path::Path;
use schemars::schema_for;
use core_access::{
    user::UserEvent,
    role::RoleEvent,
    permission_set::PermissionSetEvent,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <output_directory>", args[0]);
        std::process::exit(1);
    }

    let output_dir = &args[1];
    let output_path = Path::new(output_dir);

    if !output_path.exists() {
        fs::create_dir_all(output_path).expect("Failed to create output directory");
    }

    // Generate schema for UserEvent
    let user_schema = schema_for!(UserEvent);
    let user_schema_json = serde_json::to_string_pretty(&user_schema).unwrap();
    fs::write(
        output_path.join("user_event_schema.json"),
        user_schema_json,
    ).expect("Failed to write user_event_schema.json");

    // Generate schema for RoleEvent
    let role_schema = schema_for!(RoleEvent);
    let role_schema_json = serde_json::to_string_pretty(&role_schema).unwrap();
    fs::write(
        output_path.join("role_event_schema.json"),
        role_schema_json,
    ).expect("Failed to write role_event_schema.json");

    // Generate schema for PermissionSetEvent
    let permission_set_schema = schema_for!(PermissionSetEvent);
    let permission_set_schema_json = serde_json::to_string_pretty(&permission_set_schema).unwrap();
    fs::write(
        output_path.join("permission_set_event_schema.json"),
        permission_set_schema_json,
    ).expect("Failed to write permission_set_event_schema.json");

    println!("Generated event schemas in directory: {}", output_dir);
}