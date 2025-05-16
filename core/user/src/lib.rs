#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
pub mod event;
pub mod permission_set;
pub mod primitives;
mod publisher;
pub mod role;
pub mod user;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck as _};
use outbox::{Outbox, OutboxEventMarker};
use permission_set::PermissionSets;

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
    authz: Authorization<Audit, RoleName>,
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
            authz: authz.clone(),
            roles,
            users,
            permission_sets,
        })
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

    // BOOTSTRAP ONLY
    pub fn create_permission_set(
        &self,
        name: String,
        permissions: std::collections::HashSet<(String, String)>,
        initial_roles: &[RoleId],
    ) {
        todo!()
    }

    pub async fn add_permission_set_to_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        permission_set_id: PermissionSetId,
    ) -> Result<(), CoreUserError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreUserObject::role(role_id),
                CoreUserAction::ROLE_UPDATE,
            )
            .await?;

        let permission_set = self.permission_sets().find_by_id(permission_set_id).await?;
        let mut role = self.roles().find_by_id(role_id).await?;

        if role
            .add_permission_set(permission_set.id, audit_info)
            .did_execute()
        {
            self.roles().update(&mut role).await?;
        }

        Ok(())
    }

    pub async fn remove_permission_set_from_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        permission_set_id: PermissionSetId,
    ) -> Result<(), CoreUserError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreUserObject::role(role_id),
                CoreUserAction::ROLE_UPDATE,
            )
            .await?;

        let permission_set = self.permission_sets().find_by_id(permission_set_id).await?;
        let mut role = self.roles().find_by_id(role_id).await?;

        if role
            .remove_permission_set(permission_set.id, audit_info)
            .did_execute()
        {
            self.roles().update(&mut role).await?;
        }

        Ok(())
    }
}

impl<Audit, E> Clone for CoreUser<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            roles: self.roles.clone(),
            users: self.users.clone(),
            permission_sets: self.permission_sets.clone(),
        }
    }
}
