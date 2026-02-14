#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod bootstrap;
pub mod config;
pub mod error;
pub mod permission_set;
pub mod primitives;
pub mod public;
mod publisher;
pub mod role;
pub mod user;

use tracing::instrument;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck as _};
use es_entity::clock::ClockHandle;
use obix::out::{Outbox, OutboxEventMarker};
use permission_set::{PermissionSet, PermissionSetRepo, PermissionSetsByIdCursor};
use tracing_macros::record_error_severity;

pub use primitives::*;
pub use public::*;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::permission_set::PermissionSetEvent;
    pub use crate::role::RoleEvent;
    pub use crate::user::UserEvent;
}

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
    authz: Authorization<Audit, AuthRoleToken>,
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
    #[record_error_severity]
    #[tracing::instrument(name = "core_access.init", skip_all)]
    pub async fn init(
        pool: &sqlx::PgPool,
        config: AccessConfig,
        all_actions: Vec<ActionMapping>,
        predefined_roles: &'static [(&'static str, &'static [&'static str])],
        system_actors: &[audit::SystemActor],
        authz: &Authorization<Audit, AuthRoleToken>,
        outbox: &Outbox<E>,
        clock: ClockHandle,
    ) -> Result<Self, CoreAccessError> {
        let users = Users::init(pool, authz, outbox, clock.clone()).await?;
        let publisher = UserPublisher::new(outbox);
        let role_repo = RoleRepo::new(pool, &publisher, clock.clone());
        let permission_set_repo = PermissionSetRepo::new(pool, clock.clone());

        if let Some(email) = config.superuser_email {
            let bootstrap =
                bootstrap::Bootstrap::new(authz, &role_repo, &users, &permission_set_repo);
            bootstrap
                .bootstrap_access_control(email, all_actions, predefined_roles, system_actors)
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

    /// Creates a new user with an email and assigns the specified role.
    /// The role must exist and the user must not already exist.
    #[record_error_severity]
    #[instrument(name = "core_access.create_user", skip(self))]
    pub async fn create_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        email: impl Into<String> + std::fmt::Debug,
        role_id: impl Into<RoleId> + std::fmt::Debug,
    ) -> Result<User, CoreAccessError> {
        let role_id = role_id.into();
        let role = self.roles.find_by_id(role_id).await?;

        if role.is_superuser() {
            return Err(CoreAccessError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }

        let user = self.users.create_user(sub, email, &role).await?;
        Ok(user)
    }

    /// Creates a new role with a given name and initial permission sets. The name
    /// must be unique, an error will be raised in case of conflict.
    #[record_error_severity]
    #[tracing::instrument(name = "core_access.create_role", skip(self, permission_sets, name), fields(subject = %sub, role_name = %name, permission_sets_count = tracing::field::Empty))]
    pub async fn create_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        name: String,
        permission_sets: impl IntoIterator<Item = impl Into<PermissionSetId>>,
    ) -> Result<Role, CoreAccessError> {
        self.authz
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
        tracing::Span::current().record("permission_sets_count", permission_set_ids.len());

        self.ensure_permission_sets_exist(&permission_set_ids)
            .await?;
        let role = match self.roles.maybe_find_by_name(&name).await? {
            Some(existing) => existing,
            None => {
                let new_role = NewRole::builder()
                    .id(RoleId::new())
                    .name(name)
                    .initial_permission_sets(permission_set_ids.clone().into_iter().collect())
                    .build()
                    .expect("all fields for new role provided");
                self.roles.create(new_role).await?
            }
        };

        for permission_set_id in permission_set_ids.into_iter() {
            self.authz
                .add_role_hierarchy(role.id, permission_set_id)
                .await?;
        }

        Ok(role)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core_access.add_permission_sets_to_role", skip(self, role_id, permission_set_ids), fields(subject = %sub, role_id = tracing::field::Empty, permission_sets_count = tracing::field::Empty))]
    pub async fn add_permission_sets_to_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: impl Into<RoleId>,
        permission_set_ids: impl IntoIterator<Item = impl Into<PermissionSetId>>,
    ) -> Result<Role, CoreAccessError> {
        let role_id = role_id.into();
        tracing::Span::current().record("role_id", role_id.to_string());

        self.authz
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
        tracing::Span::current().record("permission_sets_count", permission_set_ids.len());

        let mut role = self.roles.find_by_id(role_id).await?;
        let mut changed = false;
        self.ensure_permission_sets_exist(&permission_set_ids)
            .await?;
        for permission_set_id in permission_set_ids.clone() {
            if role.add_permission_set(permission_set_id).did_execute() {
                changed = true;
            }
        }

        if changed {
            self.roles.update(&mut role).await?;
            for permission_set_id in permission_set_ids.into_iter() {
                self.authz
                    .add_role_hierarchy(role.id, permission_set_id)
                    .await?;
            }
        }

        Ok(role)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core_access.remove_permission_sets_from_role", skip(self, role_id, permission_set_ids), fields(subject = %sub, role_id = tracing::field::Empty, permission_sets_count = tracing::field::Empty))]
    pub async fn remove_permission_sets_from_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        role_id: impl Into<RoleId>,
        permission_set_ids: impl IntoIterator<Item = impl Into<PermissionSetId>>,
    ) -> Result<Role, CoreAccessError> {
        let role_id = role_id.into();
        tracing::Span::current().record("role_id", role_id.to_string());

        let permission_set_ids = permission_set_ids
            .into_iter()
            .map(|id| id.into())
            .collect::<Vec<_>>();
        tracing::Span::current().record("permission_sets_count", permission_set_ids.len());

        self.authz
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
            if role.remove_permission_set(permission_set_id).did_execute() {
                changed = true;
            }
        }

        if changed {
            self.roles.update(&mut role).await?;
            for permission_set_id in permission_set_ids.into_iter() {
                self.authz
                    .remove_role_hierarchy(role.id, permission_set_id)
                    .await?;
            }
        }

        Ok(role)
    }

    #[record_error_severity]
    #[instrument(name = "access.find_role_by_name", skip(self, name), fields(subject = %sub, role_name = %name))]
    pub async fn find_role_by_name(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        name: impl AsRef<str> + std::fmt::Debug + std::fmt::Display,
    ) -> Result<Role, RoleError> {
        let name = name.as_ref().to_owned();
        tracing::Span::current().record("role_name", &name);

        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_roles(),
                CoreAccessAction::ROLE_LIST,
            )
            .await?;
        self.roles.find_by_name(name).await
    }

    #[record_error_severity]
    #[instrument(name = "core_access.update_role_of_user", skip(self))]
    pub async fn update_role_of_user(
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
                CoreAccessAction::USER_UPDATE_ROLE,
            )
            .await?;

        let role = self.roles.find_by_id(role_id).await?;

        if role.is_superuser() {
            return Err(CoreAccessError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }

        let user = self.users.update_role_of_user(sub, user_id, &role).await?;

        Ok(user)
    }

    #[record_error_severity]
    #[instrument(name = "access.list_roles", skip(self))]
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

    #[record_error_severity]
    #[instrument(name = "access.find_all_roles", skip(self))]
    pub async fn find_all_roles<T: From<Role>>(
        &self,
        ids: &[RoleId],
    ) -> Result<std::collections::HashMap<RoleId, T>, CoreAccessError> {
        Ok(self.roles.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "access.list_permission_sets", skip(self))]
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

    #[record_error_severity]
    #[tracing::instrument(name = "core_access.find_role_by_id", skip(self, id), fields(subject = %sub, role_id = tracing::field::Empty))]
    pub async fn find_role_by_id(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        id: impl Into<RoleId>,
    ) -> Result<Option<Role>, CoreAccessError> {
        let id = id.into();
        tracing::Span::current().record("role_id", id.to_string());

        self.authz
            .enforce_permission(sub, CoreAccessObject::role(id), CoreAccessAction::ROLE_READ)
            .await?;
        Ok(self.roles.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "access.find_all_permission_sets", skip(self))]
    pub async fn find_all_permission_sets<T: From<PermissionSet>>(
        &self,
        ids: &[PermissionSetId],
    ) -> Result<std::collections::HashMap<PermissionSetId, T>, CoreAccessError> {
        Ok(self.permission_sets.find_all(ids).await?)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core_access.ensure_permission_sets_exist", skip(self), fields(count = permission_set_ids.len()))]
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
