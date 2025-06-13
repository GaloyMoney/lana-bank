mod entity;
pub mod error;
mod repo;

pub use entity::{NewRole, Role};
#[cfg(feature = "json-schema")]
pub use entity::RoleEvent;
pub use error::RoleError;
pub(super) use repo::RoleRepo;

pub use repo::role_cursor::*;
