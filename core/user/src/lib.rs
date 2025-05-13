#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod event;
pub mod primitives;
mod publisher;
pub mod role;
pub mod user;

use audit::AuditSvc;
use authz::Authorization;
use outbox::{Outbox, OutboxEventMarker};

pub use event::*;
pub use primitives::*;

pub use publisher::UserPublisher;
pub use role::*;

pub struct CoreUser<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    roles: Roles<Audit, E>,
}

impl<Audit, E> CoreUser<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Action: From<CoreUserAction>,
    <Audit as AuditSvc>::Object: From<CoreUserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, RoleName>,
        outbox: &Outbox<E>,
    ) -> Self {
        let publisher = UserPublisher::new(outbox);

        Self {
            roles: Roles::new(pool, authz, &publisher),
        }
    }

    pub fn roles(&self) -> &Roles<Audit, E> {
        &self.roles
    }
}
