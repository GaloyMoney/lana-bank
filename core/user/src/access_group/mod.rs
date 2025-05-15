//! _Access Group_ is a predefined named set of permissions. Administrators with sufficient
//! permissions can assign Access Groups to [Roles](super::role).
//!
//! The main purpose of Access Groups is to group related permissions under a common name and thus
//! shield the administrator from actual permissions that can be too dynamic and have too high a granularity.
//! Access Groups are not intended to be created or deleted in a running application; they are expected
//! to be created and defined during application bootstrap and remain unchanged for its entire life.

mod entity;
