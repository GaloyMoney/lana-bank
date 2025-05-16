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

    /// Creates a new role with given name. The names must be unique,
    /// an error will be raised in case of conflict.
    pub async fn create_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        name: RoleName,
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

    /// Make role with `role_id` inherit from role with `junior_id`.
    /// Consequently, `role_id` will gain all permissions of `junior_id`.
    pub async fn inherit_from_junior(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        junior_id: RoleId,
    ) -> Result<(), RoleError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreUserObject::role(role_id),
                CoreUserAction::ROLE_UPDATE,
            )
            .await?;

        let junior = self.repo.find_by_id(&junior_id).await?;
        let mut senior = self.repo.find_by_id(&role_id).await?;

        if senior.inherit_from(&junior, audit_info).did_execute() {
            self.authz
                .add_role_hierarchy(senior.name.clone(), junior.name)
                .await?;

            self.repo.update(&mut senior).await?;
        }

        Ok(())
    }
}

impl<Audit, E> Clone for Roles<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}
