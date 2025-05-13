use audit::AuditSvc;
use authz::{Authorization, PermissionCheck as _};
use outbox::OutboxEventMarker;

use crate::{
    event::CoreUserEvent,
    primitives::{CoreUserAction, CoreUserObject, RoleId},
    publisher::UserPublisher,
    RoleName,
};

mod entity;
pub mod error;
mod repo;

pub use entity::{NewRole, Role, RoleEvent};
use error::RoleError;
use repo::RoleRepo;

#[derive(Clone)]
pub struct Roles<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    authz: Authorization<Audit, RoleName>,
    repo: RoleRepo<E>,
}

impl<Audit, E> Roles<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Action: From<CoreUserAction>,
    <Audit as AuditSvc>::Object: From<CoreUserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, RoleName>,
        publisher: &UserPublisher<E>,
    ) -> Self {
        Self {
            repo: RoleRepo::new(pool, publisher),
            authz: authz.clone(),
        }
    }

    pub async fn create_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
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
