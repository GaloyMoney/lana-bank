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

    pub async fn add_permission(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        object: impl Into<Audit::Object>,
        action: impl Into<Audit::Action>,
    ) -> Result<Role, RoleError> {
        self.authz
            .enforce_permission(
                sub,
                CoreUserObject::role(role_id),
                CoreUserAction::ROLE_UPDATE,
            )
            .await?;

        let object = object.into();
        let action = action.into();

        let mut role = self.repo.find_by_id(&role_id).await?;
        if role
            .add_permission(object.to_string(), action.to_string())
            .did_execute()
        {
            self.authz
                .add_permission_to_role(&role.name, object, action)
                .await?;
            self.repo.update(&mut role).await?;
        }

        Ok(role)
    }

    pub async fn remove_permission(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        object: impl Into<Audit::Object>,
        action: impl Into<Audit::Action>,
    ) -> Result<Role, RoleError> {
        self.authz
            .enforce_permission(
                sub,
                CoreUserObject::role(role_id),
                CoreUserAction::ROLE_UPDATE,
            )
            .await?;

        let object = object.into();
        let action = action.into();

        let mut role = self.repo.find_by_id(&role_id).await?;
        if role
            .remove_permission(object.to_string(), action.to_string())
            .did_execute()
        {
            self.authz
                .remove_permission_from_role(&role.name, object, action)
                .await?;
            self.repo.update(&mut role).await?;
        }

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
        self.authz
            .enforce_permission(
                sub,
                CoreUserObject::role(role_id),
                CoreUserAction::ROLE_UPDATE,
            )
            .await?;

        let junior = self.repo.find_by_id(&junior_id).await?;
        let mut senior = self.repo.find_by_id(&role_id).await?;

        if senior.inherit_from(&junior).did_execute() {
            self.authz
                .add_role_hierarchy(senior.name.clone(), junior.name)
                .await?;

            self.repo.update(&mut senior).await?;
        }

        Ok(())
    }
}
