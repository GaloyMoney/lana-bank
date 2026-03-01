#[macro_use]
extern crate admin_graphql_shared;

pub mod credit_config;
pub mod deposit_config;
pub mod domain_config;
pub mod schema;

pub use schema::{ConfigMutation, ConfigQuery};
