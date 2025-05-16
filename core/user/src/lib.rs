#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod permission_set;
pub mod error;
pub mod event;
pub mod primitives;
mod publisher;
pub mod role;
pub mod user;

use permission_set::PermissionSets;
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
    permission_sets: PermissionSets<Audit>,
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
        let permission_sets = PermissionSets::new(authz, pool);

        Ok(Self {
            roles,
            users,
            permission_sets: permission_sets,
        })
    }

    // BOOTSTRAP ONLY
    pub fn create_permission_set(
        &self,
        name: String,
        permissions: std::collections::HashSet<(String, String)>,
        roles: &[RoleId],
    ) {
        todo!()
    }

    pub fn add_permission_set_to_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role: RoleId,
        permission_set: PermissionSetId,
    ) -> Result<(), CoreUserError> {
        todo!()
    }

    pub fn remove_permission_set_from_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role: RoleId,
        permission_set: PermissionSetId,
    ) -> Result<(), CoreUserError> {
        todo!()
    }

    pub fn roles(&self) -> &Roles<Audit, E> {
        &self.roles
    }

    pub fn users(&self) -> &Users<Audit, E> {
        &self.users
    }

    pub fn permission_sets(&self) -> &PermissionSets<Audit> {
        &self.permission_sets
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
            permission_sets: self.permission_sets.clone(),
        }
    }
}
