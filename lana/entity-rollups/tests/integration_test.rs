use entity_rollups::is_breaking_change;
use serde_json::Value;
use std::fs;

#[test]
fn test_breaking_change_detection() {
    // Load test fixtures
    let old_schema_content = fs::read_to_string("tests/fixtures/old_schema.json")
        .expect("Failed to read old_schema.json fixture");
    let old_schema: Value =
        serde_json::from_str(&old_schema_content).expect("Failed to parse old_schema.json");

    // Test breaking change (required field removed)
    let breaking_schema_content = fs::read_to_string("tests/fixtures/new_schema_breaking.json")
        .expect("Failed to read new_schema_breaking.json fixture");
    let breaking_schema: Value = serde_json::from_str(&breaking_schema_content)
        .expect("Failed to parse new_schema_breaking.json");

    let is_breaking = is_breaking_change(&old_schema, &breaking_schema)
        .expect("Failed to check for breaking changes");
    assert!(
        is_breaking,
        "Expected to detect breaking change when required field is removed"
    );

    // Test non-breaking change (only optional field added)
    let non_breaking_schema_content =
        fs::read_to_string("tests/fixtures/new_schema_non_breaking.json")
            .expect("Failed to read new_schema_non_breaking.json fixture");
    let non_breaking_schema: Value = serde_json::from_str(&non_breaking_schema_content)
        .expect("Failed to parse new_schema_non_breaking.json");

    let is_breaking = is_breaking_change(&old_schema, &non_breaking_schema)
        .expect("Failed to check for breaking changes");
    assert!(
        !is_breaking,
        "Expected non-breaking change when only optional field is added"
    );
}

#[test]
fn test_type_change_detection() {
    let old_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "field1": {
                "type": "string"
            }
        },
        "required": ["field1"]
    });

    let new_schema_with_type_change = serde_json::json!({
        "type": "object",
        "properties": {
            "field1": {
                "type": "integer"
            }
        },
        "required": ["field1"]
    });

    let is_breaking = is_breaking_change(&old_schema, &new_schema_with_type_change)
        .expect("Failed to check for breaking changes");
    assert!(
        is_breaking,
        "Expected to detect breaking change when field type changes"
    );
}
