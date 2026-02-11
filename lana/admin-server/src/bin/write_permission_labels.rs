use std::collections::BTreeMap;

use heck::ToTitleCase;
use serde_json::json;

fn main() {
    // Reference admin_server to ensure all domain crates are linked,
    // which registers their permission set entries via inventory.
    let _ = admin_server::graphql::schema(None);

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
    let root = json!({ "Permissions": permissions });
    println!("{}", serde_json::to_string_pretty(&root).unwrap());
}
