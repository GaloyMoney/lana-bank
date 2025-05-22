//! _Permission Set_ is a predefined named set of permissions. Administrators with sufficient
//! permissions can assign Permission Sets to a [Role](super::role) and thus give the users
//! with this role all permissions of the Permission Set.
//!
//! The main purpose of Permission Sets is to group related permissions under a common name and
//! shield the administrator from actual permissions that can be too dynamic and have too high a granularity.
//! Permission Sets are defined / created and modified during startup of the runtime. No mutating
//! operations are exposed to the outside world.

mod entity;
mod error;
mod repo;

pub(crate) use entity::NewPermissionSet;
pub use entity::PermissionSet;
pub use error::PermissionSetError;
pub(super) use repo::PermissionSetRepo;
