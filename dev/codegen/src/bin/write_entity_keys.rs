use std::collections::BTreeMap;

fn main() {
    let schema = include_str!("../../../../lana/admin-server/src/graphql/schema.graphql");

    let mut entity_keys: BTreeMap<&str, &str> = BTreeMap::new();

    for line in schema.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("type ") else {
            continue;
        };
        let Some(directive_start) = rest.find("@entity_key(field: \"") else {
            continue;
        };
        let type_name = rest[..directive_start].split_whitespace().next().unwrap();
        let field_start = directive_start + "@entity_key(field: \"".len();
        if let Some(field_end) = rest[field_start..].find('"') {
            let field_name = &rest[field_start..field_start + field_end];
            entity_keys.insert(type_name, field_name);
        }
    }

    println!("// @generated - do not edit. Run `make generate-entity-keys` to regenerate.");
    println!();
    println!("export const entityKeyFields: Record<string, readonly string[]> = {{");
    for (type_name, field_name) in &entity_keys {
        println!("  {type_name}: [\"{field_name}\"],");
    }
    println!("}}");
}
