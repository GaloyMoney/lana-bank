#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod event;
pub mod primitives;
mod publisher;
pub mod role;
pub mod user;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

pub use event::*;
pub use primitives::*;

use publisher::UserPublisher;
pub use role::*;

pub struct CoreUser<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreUserEvent>,
{
    roles: Roles<Perms, E>,
}

impl<Perms, E> CoreUser<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreUserAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreUserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub async fn init(pool: &sqlx::PgPool, authz: &Perms, outbox: &Outbox<E>) -> Self {
        let publisher = UserPublisher::new(outbox);

        Self {
            roles: Roles::new(pool, authz, &publisher),
        }
    }

    pub fn roles(&self) -> &Roles<Perms, E> {
        &self.roles
    }
}
