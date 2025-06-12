use serde_json::Value;

pub fn is_breaking_change(old_schema: &Value, new_schema: &Value) -> anyhow::Result<bool> {
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
