use async_graphql::*;

#[TypeDirective(location = "Object")]
pub fn entity_key(field: String) {}
