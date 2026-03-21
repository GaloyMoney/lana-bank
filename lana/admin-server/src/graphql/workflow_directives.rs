use async_graphql::*;

#[TypeDirective(location = "FieldDefinition", repeatable)]
pub fn workflow_require(token: String) {}

#[TypeDirective(location = "FieldDefinition", repeatable)]
pub fn workflow_output(path: String) {}
