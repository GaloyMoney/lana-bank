// Minimal standalone binary to generate JSON schema for UserEvent
use schemars::schema_for;
use core_access::user::UserEvent;

fn main() {
    let schema = schema_for!(UserEvent);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}