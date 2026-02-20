#[macro_use]
extern crate admin_graphql_shared;

pub mod primitives;

pub mod customer;
pub mod document;
pub mod prospect;
pub mod schema;

pub use customer::*;
pub use document::*;
pub use prospect::*;
pub use schema::*;
