//! _Permission Set_ is a predefined named set of permissions. Administrators with sufficient
//! permissions can assign Permission Sets to a [Role](super::role) and thus give the users
//! with this role all permissions of the Permission Set.
//!
//! The main purpose of Permission Sets is to group related permissions under a common name and
//! shield the administrator from actual permissions that can be too dynamic and have too high a granularity.
//! Permission Sets are defined / created and modified during startup of the runtime. No mutating
//! operations are exposed to the outside world.

mod entity;
pub mod error;
mod repo;

pub(crate) use entity::NewPermissionSet;
pub(super) use error::PermissionSetError;
pub(super) use repo::PermissionSetRepo;

pub use entity::PermissionSet;
pub use repo::permission_set_cursor::*;

#[cfg(feature = "json-schema")]
pub use entity::PermissionSetEvent;
