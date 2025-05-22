mod entity;
pub mod error;
mod repo;

pub use entity::{NewRole, Role, RoleEvent};
pub use error::RoleError;
pub(super) use repo::RoleRepo;
