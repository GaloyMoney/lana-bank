mod entity;
pub mod error;
mod repo;

pub use entity::{NewRole, Role};
// RoleEvent is available internally and conditionally publicly
#[cfg(feature = "json-schema")]
pub use entity::RoleEvent;
#[cfg(not(feature = "json-schema"))]
pub(crate) use entity::RoleEvent;
pub use error::RoleError;
pub(super) use repo::RoleRepo;

pub use repo::role_cursor::*;
