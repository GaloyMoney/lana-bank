use async_graphql::*;

#[TypeDirective(location = "ArgumentDefinition", location = "InputFieldDefinition")]
pub fn entity_ref(#[graphql(name = "type")] type_name: String) {}

#[TypeDirective(location = "FieldDefinition", repeatable)]
pub fn workflow_require(token: String) {}

#[TypeDirective(location = "FieldDefinition", repeatable)]
pub fn workflow_output(path: String) {}
