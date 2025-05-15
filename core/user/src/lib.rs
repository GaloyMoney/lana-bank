#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
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
pub use user::*;

use error::CoreUserError;

pub struct CoreUser<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    roles: Roles<Audit, E>,
    users: Users<Audit, E>,
}

impl<Audit, E> CoreUser<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreUserAction>,
    <Audit as AuditSvc>::Object: From<CoreUserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, RoleName>,
        outbox: &Outbox<E>,
        superuser_email: Option<String>,
    ) -> Result<Self, CoreUserError> {
        let users = Users::init(pool, authz, outbox, superuser_email).await?;
        let publisher = UserPublisher::new(outbox);
        let roles = Roles::new(pool, authz, &publisher);

        Ok(Self { roles, users })
    }

    pub fn roles(&self) -> &Roles<Audit, E> {
        &self.roles
    }

    pub fn users(&self) -> &Users<Audit, E> {
        &self.users
    }
}

impl<Audit, E> Clone for CoreUser<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    fn clone(&self) -> Self {
        Self {
            roles: self.roles.clone(),
            users: self.users.clone(),
        }
    }
}
