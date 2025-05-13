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
    authz: Authorization<Audit, String>,
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
        authz: &Authorization<Audit, String>,
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

    pub async fn assign_to_parent(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        parent_id: RoleId,
    ) -> Result<(), RoleError> {
        self.authz
            .enforce_permission(
                sub,
                CoreUserObject::all_roles(),
                CoreUserAction::ROLE_UPDATE,
            )
            .await?;

        let mut roles = self.repo.find_all::<Role>(&[role_id, parent_id]).await?;

        let mut child = roles.remove(&role_id).expect("role was found");
        let parent = roles.remove(&parent_id).expect("parent was found");

        if child.assign_to_parent(&parent).did_execute() {
            self.authz
                .add_role_hierarchy(parent.name, child.name.clone())
                .await?;

            self.repo.update(&mut child).await?;
        }

        Ok(())
    }
}
