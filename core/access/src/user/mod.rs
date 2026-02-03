mod entity;
pub mod error;
mod repo;

use es_entity::{DbOp, Idempotent};
use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck};
use obix::out::{Outbox, OutboxEventMarker};
use tracing_macros::record_error_severity;

use crate::{Role, primitives::*, public::*, publisher::UserPublisher};

pub use entity::User;
use entity::*;
// UserEvent is available internally and conditionally publicly
#[cfg(feature = "json-schema")]
pub use entity::UserEvent;
#[cfg(not(feature = "json-schema"))]
pub(crate) use entity::UserEvent;
pub use error::*;
pub use repo::user_cursor::UsersByCreatedAtCursor;
use repo::*;

pub struct Users<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    authz: Authorization<Audit, AuthRoleToken>,
    repo: UserRepo<E>,
}

impl<Audit, E> Clone for Users<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Audit, E> Users<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, AuthRoleToken>,
        outbox: &Outbox<E>,
        clock: es_entity::clock::ClockHandle,
    ) -> Result<Self, UserError> {
        let publisher = UserPublisher::new(outbox);
        let repo = UserRepo::new(pool, &publisher, clock);

        Ok(Self {
            repo,
            authz: authz.clone(),
        })
    }

    pub async fn subject_can_create_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_CREATE,
                enforce,
            )
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_access.create_user", skip(self, role))]
    pub async fn create_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        email: impl Into<String> + std::fmt::Debug,
        role: &Role,
    ) -> Result<User, UserError> {
        self.subject_can_create_user(sub, true)
            .await?
            .expect("audit info missing");

        let email = email.into();

        let mut db = self.repo.begin_op().await?;

        let new_user = NewUser::builder()
            .email(email.clone())
            .role_id(role.id)
            .build()
            .expect("Could not build user");
        let user = self.repo.create_in_op(&mut db, new_user).await?;

        // Assign the role in the authorization system
        self.authz.assign_role_to_subject(user.id, role.id).await?;

        db.commit().await?;

        Ok(user)
    }

    #[record_error_severity]
    #[instrument(name = "core_access.find_for_subject", skip(self))]
    pub async fn find_for_subject(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
    ) -> Result<User, UserError>
    where
        UserId: for<'a> TryFrom<&'a <Audit as AuditSvc>::Subject>,
    {
        let id = UserId::try_from(sub).map_err(|_| UserError::SubjectIsNotUser)?;
        self.authz
            .enforce_permission(sub, CoreAccessObject::user(id), CoreAccessAction::USER_READ)
            .await?;
        self.repo.find_by_id(id).await
    }

    #[record_error_severity]
    #[instrument(name = "core_access.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        id: impl Into<UserId> + std::fmt::Debug,
    ) -> Result<Option<User>, UserError> {
        let id = id.into();
        self.authz
            .enforce_permission(sub, CoreAccessObject::user(id), CoreAccessAction::USER_READ)
            .await?;
        self.repo.maybe_find_by_id(id).await
    }

    #[record_error_severity]
    #[instrument(name = "core_access.find_by_email", skip(self))]
    pub async fn find_by_email(
        &self,
        sub: Option<&<Audit as AuditSvc>::Subject>,
        email: &String,
    ) -> Result<Option<User>, UserError> {
        if let Some(sub) = sub {
            self.authz
                .enforce_permission(
                    sub,
                    CoreAccessObject::all_users(),
                    CoreAccessAction::USER_READ,
                )
                .await?;
        }

        self.repo.maybe_find_by_email(email.as_str()).await
    }

    #[record_error_severity]
    #[instrument(name = "core_access.find_all", skip(self))]
    pub async fn find_all<T: From<User>>(
        &self,
        ids: &[UserId],
    ) -> Result<HashMap<UserId, T>, UserError> {
        self.repo.find_all(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "core_access.list_users", skip(self))]
    pub async fn list_users(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
    ) -> Result<Vec<User>, UserError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_LIST,
            )
            .await?;

        Ok(self
            .repo
            .list_by_email(Default::default(), es_entity::ListDirection::Ascending)
            .await?
            .entities)
    }

    pub async fn list_users_without_audit(
        &self,
        query: es_entity::PaginatedQueryArgs<UsersByCreatedAtCursor>,
        direction: es_entity::ListDirection,
    ) -> Result<es_entity::PaginatedQueryRet<User, UsersByCreatedAtCursor>, UserError> {
        self.repo.list_by_created_at(query, direction).await
    }

    pub async fn subject_can_update_role_of_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<Option<UserId>>,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreAccessObject::user(user_id),
                CoreAccessAction::USER_UPDATE_ROLE,
                enforce,
            )
            .await?)
    }

    pub(crate) async fn update_role_of_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<UserId> + std::fmt::Debug,
        role: &Role,
    ) -> Result<User, UserError> {
        let id = user_id.into();

        self.subject_can_update_role_of_user(sub, id, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;

        if let Idempotent::Executed(previous) = user.update_role(role) {
            self.authz
                .revoke_role_from_subject(user.id, previous)
                .await?;
            self.authz.assign_role_to_subject(user.id, role.id).await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }

    /// Creates a user with `email` and belonging to `role` (superuser).
    /// Used for bootstrapping the application.
    pub(super) async fn bootstrap_superuser_user_in_op(
        &self,
        op: &mut DbOp<'_>,
        email: String,
        role: &Role,
    ) -> Result<User, UserError> {
        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut *op,
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_CREATE,
            )
            .await?;

        let user = match self
            .repo
            .maybe_find_by_email_in_op(&mut *op, &email)
            .await?
        {
            None => {
                let new_user = NewUser::builder()
                    .id(UserId::new())
                    .email(email)
                    .role_id(role.id)
                    .build()
                    .expect("all fields for new user provided");

                self.repo.create_in_op(&mut *op, new_user).await?
            }
            Some(mut user) => {
                // Update existing user's role if needed
                if user.update_role(role).did_execute() {
                    self.repo.update_in_op(op, &mut user).await?;
                }
                user
            }
        };

        Ok(user)
    }
}
