#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod bootstrap;
pub mod config;
pub mod error;
pub mod event;
pub mod permission_set;
pub mod primitives;
mod publisher;
pub mod role;
pub mod user;

use tracing::instrument;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck as _};
use outbox::{Outbox, OutboxEventMarker};
use permission_set::{PermissionSet, PermissionSetRepo, PermissionSetsByIdCursor};

pub use event::*;
pub use primitives::*;

use config::AccessConfig;
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
        config: AccessConfig,
        authz: &Authorization<Audit, RoleName>,
        outbox: &Outbox<E>,
    ) -> Result<Self, CoreAccessError> {
        let users = Users::init(pool, authz, outbox).await?;
        let publisher = UserPublisher::new(outbox);
        let role_repo = RoleRepo::new(pool, &publisher);
        let permission_set_repo = PermissionSetRepo::new(pool);

        if let Some(email) = config.superuser_email {
            let bootstrap =
                bootstrap::Bootstrap::new(authz, &role_repo, &users, &permission_set_repo);
            bootstrap
                .bootstrap_access_control(
                    email,
                    config.action_descriptions,
                    config.predefined_roles,
                )
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
        permission_sets: impl IntoIterator<Item = impl Into<PermissionSetId>>,
    ) -> Result<Role, CoreAccessError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_roles(),
                CoreAccessAction::ROLE_CREATE,
            )
            .await?;

        let permission_set_ids = permission_sets
            .into_iter()
            .map(|id| id.into())
            .collect::<Vec<_>>();
        self.ensure_permission_sets_exist(&permission_set_ids)
            .await?;
        let new_role = NewRole::builder()
            .id(RoleId::new())
            .name(name)
            .initial_permission_sets(permission_set_ids.into_iter().collect())
            .audit_info(audit_info)
            .build()
            .expect("all fields for new role provided");

        Ok(self.roles.create(new_role).await?)
    }

    pub async fn add_permission_sets_to_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: impl Into<RoleId>,
        permission_set_ids: impl IntoIterator<Item = impl Into<PermissionSetId>>,
    ) -> Result<Role, CoreAccessError> {
        let role_id = role_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccessObject::role(role_id),
                CoreAccessAction::ROLE_UPDATE,
            )
            .await?;

        let permission_set_ids = permission_set_ids
            .into_iter()
            .map(|id| id.into())
            .collect::<Vec<_>>();

        let mut role = self.roles.find_by_id(role_id).await?;
        let mut changed = false;
        self.ensure_permission_sets_exist(&permission_set_ids)
            .await?;
        for permission_set_id in permission_set_ids {
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

        Ok(role)
    }

    pub async fn remove_permission_sets_from_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: impl Into<RoleId>,
        permission_set_ids: impl IntoIterator<Item = impl Into<PermissionSetId>>,
    ) -> Result<Role, CoreAccessError> {
        let role_id = role_id.into();
        let permission_set_ids = permission_set_ids
            .into_iter()
            .map(|id| id.into())
            .collect::<Vec<_>>();

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
            .find_all::<PermissionSet>(&permission_set_ids)
            .await?;

        let mut changed = false;

        for (permission_set_id, _) in permission_sets {
            if role
                .remove_permission_set(permission_set_id, audit_info.clone())
                .did_execute()
            {
                changed = true;
            }
        }

        if changed {
            self.roles.update(&mut role).await?;
        }

        Ok(role)
    }

    #[instrument(name = "access.find_role_by_name", skip(self), err)]
    pub async fn find_role_by_name(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        name: RoleName,
    ) -> Result<Role, RoleError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_roles(),
                CoreAccessAction::ROLE_LIST,
            )
            .await?;
        self.roles.find_by_name(name).await
    }

    #[instrument(name = "core_access.assign_role_to_user", skip(self))]
    pub async fn assign_role_to_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<UserId> + std::fmt::Debug,
        role_id: impl Into<RoleId> + std::fmt::Debug,
    ) -> Result<User, CoreAccessError> {
        let user_id = user_id.into();
        let role_id = role_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::user(user_id),
                CoreAccessAction::USER_ASSIGN_ROLE,
            )
            .await?;

        let role = self.roles.find_by_id(role_id).await?;
        let user = self.users.assign_role_to_user(sub, user_id, &role).await?;

        Ok(user)
    }

    #[instrument(name = "core_access.revoke_role_from_user", skip(self))]
    pub async fn revoke_role_from_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<UserId> + std::fmt::Debug,
        role_id: impl Into<RoleId> + std::fmt::Debug,
    ) -> Result<User, CoreAccessError> {
        let user_id = user_id.into();
        let role_id = role_id.into();

        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::user(user_id),
                CoreAccessAction::USER_REVOKE_ROLE,
            )
            .await?;

        let role = self.roles.find_by_id(role_id).await?;
        let user = self
            .users
            .revoke_role_from_user(sub, user_id, &role)
            .await?;

        Ok(user)
    }

    #[instrument(name = "access.list_roles", skip(self), err)]
    pub async fn list_roles(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<RolesByNameCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Role, RolesByNameCursor>, CoreAccessError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_roles(),
                CoreAccessAction::ROLE_LIST,
            )
            .await?;
        Ok(self
            .roles
            .list_by_name(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[instrument(name = "access.find_all_roles", skip(self), err)]
    pub async fn find_all_roles<T: From<Role>>(
        &self,
        ids: &[RoleId],
    ) -> Result<std::collections::HashMap<RoleId, T>, CoreAccessError> {
        Ok(self.roles.find_all(ids).await?)
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

    pub async fn find_role_by_id(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        id: impl Into<RoleId>,
    ) -> Result<Option<Role>, CoreAccessError> {
        let id = id.into();
        self.authz
            .enforce_permission(sub, CoreAccessObject::role(id), CoreAccessAction::ROLE_READ)
            .await?;
        match self.roles.find_by_id(id).await {
            Ok(role) => Ok(Some(role)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "access.list_roles", skip(self), err)]
    pub async fn list_roles(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<RolesByNameCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Role, RolesByNameCursor>, CoreAccessError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_roles(),
                CoreAccessAction::ROLE_LIST,
            )
            .await?;
        Ok(self
            .roles
            .list_by_name(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[instrument(name = "access.find_all_permission_sets", skip(self), err)]
    pub async fn find_all_permission_sets<T: From<PermissionSet>>(
        &self,
        ids: &[PermissionSetId],
    ) -> Result<std::collections::HashMap<PermissionSetId, T>, CoreAccessError> {
        Ok(self.permission_sets.find_all(ids).await?)
    }

    async fn ensure_permission_sets_exist(
        &self,
        permission_set_ids: &[PermissionSetId],
    ) -> Result<(), CoreAccessError> {
        let permission_sets = self
            .permission_sets
            .find_all::<PermissionSet>(permission_set_ids)
            .await?;
        for id in permission_set_ids {
            if !permission_sets.contains_key(id) {
                return Err(CoreAccessError::PermissionSetError(
                    permission_set::PermissionSetError::EsEntityError(
                        es_entity::EsEntityError::NotFound,
                    ),
                ));
            }
        }

        Ok(())
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
