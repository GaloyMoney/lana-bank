use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::OutboxEventMarker;

use crate::{
    event::CoreUserEvent,
    primitives::{CoreUserAction, CoreUserObject, RoleId},
    publisher::UserPublisher,
};

mod entity;
pub mod error;
mod repo;

pub use entity::{NewRole, Role, RoleEvent};
use error::RoleError;
use repo::RoleRepo;

#[derive(Clone)]
pub struct Roles<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreUserEvent>,
{
    authz: Perms,
    repo: RoleRepo<E>,
}

impl<Perms, E> Roles<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreUserAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreUserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, publisher: &UserPublisher<E>) -> Self {
        Self {
            repo: RoleRepo::new(pool, publisher),
            authz: authz.clone(),
        }
    }

    pub async fn create_role(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
    ) -> Result<Role, RoleError> {
        self.authz
            .enforce_permission(
                sub,
                CoreUserObject::all_roles(),
                CoreUserAction::ROLE_CREATE,
            )
            .await?;

        let new_role = NewRole::builder()
            .id(RoleId::new())
            .name(name)
            .build()
            .expect("all fields for new role provided");

        let role = self.repo.create(new_role).await?;

        Ok(role)
    }
}
