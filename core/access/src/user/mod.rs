mod entity;
pub mod error;
mod repo;

use es_entity::DbOp;
use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck};
use outbox::{Outbox, OutboxEventMarker};

use crate::{event::*, primitives::*, publisher::UserPublisher, Role};

use entity::*;
pub use entity::{User, UserEvent};
pub use error::*;
use repo::*;

pub struct Users<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    authz: Authorization<Audit, RoleName>,
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
        authz: &Authorization<Audit, RoleName>,
        outbox: &Outbox<E>,
    ) -> Result<Self, UserError> {
        let publisher = UserPublisher::new(outbox);
        let repo = UserRepo::new(pool, &publisher);

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

    #[instrument(name = "core_access.create_user", skip(self))]
    pub async fn create_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        email: impl Into<String> + std::fmt::Debug,
    ) -> Result<User, UserError> {
        let audit_info = self
            .subject_can_create_user(sub, true)
            .await?
            .expect("audit info missing");

        let email = email.into();

        let new_user = NewUser::builder()
            .email(email.clone())
            .audit_info(audit_info)
            .build()
            .expect("Could not build user");
        let user = self.repo.create(new_user).await?;
        Ok(user)
    }

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
        match self.repo.find_by_id(id).await {
            Ok(user) => Ok(Some(user)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

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

        match self.repo.find_by_email(email).await {
            Ok(user) => Ok(Some(user)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "core_access.update_authentication_id_for_user", skip(self))]
    pub async fn update_authentication_id_for_user(
        &self,
        user_id: UserId,
        authentication_id: AuthenticationId,
    ) -> Result<User, UserError> {
        self.authz
            .audit()
            .record_system_entry(
                CoreAccessObject::user(user_id),
                CoreAccessAction::USER_UPDATE_AUTHENTICATION_ID,
            )
            .await?;

        let mut user = self.repo.find_by_id(user_id).await?;
        if user
            .update_authentication_id(authentication_id)
            .did_execute()
        {
            self.repo.update(&mut user).await?;
        }
        Ok(user)
    }

    #[instrument(
        name = "core_access.find_by_authentication_id",
        skip(self, authentication_id)
    )]
    pub async fn find_by_authentication_id(
        &self,
        authentication_id: AuthenticationId,
    ) -> Result<User, UserError> {
        self.repo
            .find_by_authentication_id(Some(authentication_id))
            .await
    }

    #[instrument(name = "core_access.find_all", skip(self))]
    pub async fn find_all<T: From<User>>(
        &self,
        ids: &[UserId],
    ) -> Result<HashMap<UserId, T>, UserError> {
        self.repo.find_all(ids).await
    }

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

    pub async fn subject_can_assign_role_to_user(
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
                CoreAccessAction::USER_ASSIGN_ROLE,
                enforce,
            )
            .await?)
    }

    pub(crate) async fn assign_role_to_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<UserId> + std::fmt::Debug,
        role: &Role,
    ) -> Result<User, UserError> {
        let id = user_id.into();

        if role.name == RoleName::SUPERUSER {
            return Err(UserError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }

        let audit_info = self
            .subject_can_assign_role_to_user(sub, id, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        let old_role = user.current_role();
        if user.assign_role(role, audit_info).did_execute() {
            if let Some(old_role_id) = old_role {
                self.authz
                    .revoke_role_from_subject(user.id, old_role_id)
                    .await?;
            }
            self.authz.assign_role_to_subject(user.id, role.id).await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }

    pub async fn subject_can_revoke_role_from_user(
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
                CoreAccessAction::USER_REVOKE_ROLE,
                enforce,
            )
            .await?)
    }

    pub async fn revoke_role_from_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<UserId> + std::fmt::Debug,
        role: &Role,
    ) -> Result<User, UserError> {
        let id = user_id.into();

        if role.name == RoleName::SUPERUSER {
            return Err(UserError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }
        let audit_role = self
            .subject_can_revoke_role_from_user(sub, id, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        if user.revoke_role(role, audit_role).did_execute() {
            self.authz
                .revoke_role_from_subject(user.id, role.id)
                .await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }

    /// Creates a user with `email` and belonging to `role` (superuser).
    /// Used for bootstrapping the application.
    pub(super) async fn bootstrap_superuser_user(
        &self,
        db: &mut DbOp<'_>,
        email: String,
        role: &Role,
    ) -> Result<User, UserError> {
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreAccessObject::all_users(),
                CoreAccessAction::USER_CREATE,
            )
            .await?;

        let user = match self.repo.find_by_email_in_tx(db.tx(), &email).await {
            Err(e) if e.was_not_found() => {
                let new_user = NewUser::builder()
                    .id(UserId::new())
                    .email(email)
                    .audit_info(audit_info.clone())
                    .build()
                    .expect("all fields for new user provided");

                let mut user = self.repo.create_in_op(db, new_user).await?;

                if user.assign_role(role, audit_info).did_execute() {
                    self.repo.update_in_op(db, &mut user).await?;
                }

                user
            }
            Err(e) => return Err(e),
            Ok(mut user) => {
                if user.assign_role(role, audit_info).did_execute() {
                    self.repo.update_in_op(db, &mut user).await?;
                };

                user
            }
        };

        Ok(user)
    }
}
