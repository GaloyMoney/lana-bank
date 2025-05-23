#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod bootstrap;
pub mod error;
pub mod event;
pub mod permission_set;
pub mod primitives;
mod publisher;
pub mod role;
pub mod user;

use std::collections::HashSet;
use tracing::instrument;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck as _};
use outbox::{Outbox, OutboxEventMarker};
use permission_set::{PermissionSet, PermissionSetRepo, PermissionSetsByIdCursor};

pub use event::*;
pub use primitives::*;

pub use publisher::UserPublisher;
pub use role::*;
pub use user::*;

use error::CoreAccessError;

pub struct CoreAccess<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    authz: Authorization<Audit, RoleName>,
    users: Users<Audit, E>,
    roles: RoleRepo<E>,
    permission_sets: PermissionSetRepo,
}

impl<Audit, E> CoreAccess<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, RoleName>,
        outbox: &Outbox<E>,
        superuser_email: Option<String>,
        action_descriptions: &[ActionDescription<FullPath>],
    ) -> Result<Self, CoreAccessError> {
        let users = Users::init(pool, authz, outbox).await?;
        let publisher = UserPublisher::new(outbox);
        let role_repo = RoleRepo::new(pool, &publisher);
        let permission_set_repo = PermissionSetRepo::new(pool);

        if let Some(email) = superuser_email {
            let bootstrap =
                bootstrap::Bootstrap::new(authz, &role_repo, &users, &permission_set_repo);
            bootstrap
                .bootstrap_access_control(email, action_descriptions)
                .await?;
        }

        let core_access = Self {
            authz: authz.clone(),
            users,
            roles: role_repo,
            permission_sets: permission_set_repo,
        };

        Ok(core_access)
    }

    pub fn users(&self) -> &Users<Audit, E> {
        &self.users
    }

    /// Creates a new role with a given name and initial permission sets. The name
    /// must be unique, an error will be raised in case of conflict.
    pub async fn create_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        name: RoleName,
        permission_sets: HashSet<PermissionSetId>,
    ) -> Result<Role, RoleError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_roles(),
                CoreAccessAction::ROLE_CREATE,
            )
            .await?;

        let new_role = NewRole::builder()
            .id(RoleId::new())
            .name(name)
            .permission_sets(permission_sets)
            .audit_info(audit_info)
            .build()
            .expect("all fields for new role provided");

        self.roles.create(new_role).await
    }

    pub async fn add_permission_sets_to_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        permission_set_ids: &[PermissionSetId],
    ) -> Result<(), CoreAccessError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccessObject::role(role_id),
                CoreAccessAction::ROLE_UPDATE,
            )
            .await?;

        let mut role = self.roles.find_by_id(role_id).await?;
        let permission_sets = self
            .permission_sets
            .find_all::<PermissionSet>(permission_set_ids)
            .await?;

        let mut changed = false;

        for (permission_set_id, _) in permission_sets {
            if role
                .add_permission_set(permission_set_id, audit_info.clone())
                .did_execute()
            {
                changed = true;
            }
        }

        if changed {
            self.roles.update(&mut role).await?;
        }

        Ok(())
    }

    pub async fn remove_permission_set_from_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: RoleId,
        permission_set_id: PermissionSetId,
    ) -> Result<(), CoreAccessError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccessObject::role(role_id),
                CoreAccessAction::ROLE_UPDATE,
            )
            .await?;

        let permission_set = self.permission_sets.find_by_id(permission_set_id).await?;
        let mut role = self.roles.find_by_id(role_id).await?;

        if role
            .remove_permission_set(permission_set.id, audit_info)
            .did_execute()
        {
            self.roles.update(&mut role).await?;
        }

        Ok(())
    }

    #[instrument(name = "access.list_permission_sets", skip(self), err)]
    pub async fn list_permission_sets(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<PermissionSetsByIdCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<PermissionSet, PermissionSetsByIdCursor>,
        CoreAccessError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_permission_sets(),
                CoreAccessAction::PERMISSION_SET_LIST,
            )
            .await?;
        Ok(self
            .permission_sets
            .list_by_id(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[instrument(name = "access.find_all_permission_sets", skip(self), err)]
    pub async fn find_all_permission_sets<T: From<PermissionSet>>(
        &self,
        ids: &[PermissionSetId],
    ) -> Result<std::collections::HashMap<PermissionSetId, T>, CoreAccessError> {
        Ok(self.permission_sets.find_all(ids).await?)
    }
}

impl<Audit, E> Clone for CoreAccess<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            users: self.users.clone(),
            roles: self.roles.clone(),
            permission_sets: self.permission_sets.clone(),
        }
    }
}
