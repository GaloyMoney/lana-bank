#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod event;
mod primitives;
mod repo;

use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck};
use outbox::{Outbox, OutboxEventMarker};

pub use entity::User;
use entity::*;
use error::*;
pub use event::*;
pub use primitives::*;
use repo::*;

pub struct Users<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    authz: Authorization<Audit, RoleName>,
    outbox: Outbox<E>,
    repo: UserRepo,
}

impl<Audit, E> Clone for Users<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Audit, E> Users<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreUserAction>,
    <Audit as AuditSvc>::Object: From<UserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, RoleName>,
        outbox: &Outbox<E>,
        superuser_email: Option<String>,
    ) -> Result<Self, UserError> {
        let repo = UserRepo::new(pool);
        let users = Self {
            repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
        };

        if let Some(email) = superuser_email {
            users.create_and_assign_role_to_superuser(email).await?;
        }

        Ok(users)
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
                UserObject::all_users(),
                CoreUserAction::USER_CREATE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "core_user.create_user", skip(self))]
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
        let mut db = self.repo.begin_op().await?;
        let user = self.repo.create_in_op(&mut db, new_user).await?;
        self.outbox
            .publish_persisted(db.tx(), CoreUserEvent::UserCreated { id: user.id, email })
            .await?;
        db.commit().await?;
        Ok(user)
    }

    #[instrument(name = "core_user.find_for_subject", skip(self))]
    pub async fn find_for_subject(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
    ) -> Result<User, UserError>
    where
        UserId: for<'a> TryFrom<&'a <Audit as AuditSvc>::Subject>,
    {
        let id = UserId::try_from(sub).map_err(|_| UserError::SubjectIsNotUser)?;
        self.authz
            .enforce_permission(sub, UserObject::user(id), CoreUserAction::USER_READ)
            .await?;
        self.repo.find_by_id(id).await
    }

    #[instrument(name = "core_user.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        id: impl Into<UserId> + std::fmt::Debug,
    ) -> Result<Option<User>, UserError> {
        let id = id.into();
        self.authz
            .enforce_permission(sub, UserObject::user(id), CoreUserAction::USER_READ)
            .await?;
        match self.repo.find_by_id(id).await {
            Ok(user) => Ok(Some(user)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "core_user.find_by_email", skip(self))]
    pub async fn find_by_email(
        &self,
        sub: Option<&<Audit as AuditSvc>::Subject>,
        email: &String,
    ) -> Result<Option<User>, UserError> {
        if let Some(sub) = sub {
            self.authz
                .enforce_permission(sub, UserObject::all_users(), CoreUserAction::USER_READ)
                .await?;
        }

        match self.repo.find_by_email(email).await {
            Ok(user) => Ok(Some(user)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "core_user.update_authentication_id_for_user", skip(self))]
    pub async fn update_authentication_id_for_user(
        &self,
        user_id: UserId,
        authentication_id: AuthenticationId,
    ) -> Result<User, UserError> {
        self.authz
            .audit()
            .record_system_entry(
                UserObject::user(user_id),
                CoreUserAction::USER_UPDATE_AUTHENTICATION_ID,
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
        name = "core_user.find_by_authentication_id",
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

    #[instrument(name = "core_user.find_all", skip(self))]
    pub async fn find_all<T: From<User>>(
        &self,
        ids: &[UserId],
    ) -> Result<HashMap<UserId, T>, UserError> {
        self.repo.find_all(ids).await
    }

    #[instrument(name = "core_user.list_users", skip(self))]
    pub async fn list_users(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
    ) -> Result<Vec<User>, UserError> {
        self.authz
            .enforce_permission(sub, UserObject::all_users(), CoreUserAction::USER_LIST)
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
                UserObject::user(user_id),
                CoreUserAction::USER_ASSIGN_ROLE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "core_user.assign_role_to_user", skip(self))]
    pub async fn assign_role_to_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<UserId> + std::fmt::Debug,
        role: impl Into<RoleName> + std::fmt::Debug,
    ) -> Result<User, UserError> {
        let id = user_id.into();
        let role = role.into();

        if role == RoleName::SUPERUSER {
            return Err(UserError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }
        let audit_info = self
            .subject_can_assign_role_to_user(sub, id, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        if user.assign_role(role.clone(), audit_info).did_execute() {
            self.authz.assign_role_to_subject(user.id, role).await?;
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
                UserObject::user(user_id),
                CoreUserAction::USER_REVOKE_ROLE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "core_user.revoke_role_from_user", skip(self))]
    pub async fn revoke_role_from_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: impl Into<UserId> + std::fmt::Debug,
        role: impl Into<RoleName> + std::fmt::Debug,
    ) -> Result<User, UserError> {
        let id = user_id.into();
        let role = role.into();

        if role == RoleName::SUPERUSER {
            return Err(UserError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }
        let audit_role = self
            .subject_can_revoke_role_from_user(sub, id, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        if user.revoke_role(role.clone(), audit_role).did_execute() {
            self.authz.revoke_role_from_subject(user.id, role).await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }

    async fn create_and_assign_role_to_superuser(&self, email: String) -> Result<(), UserError> {
        let mut db = self.repo.begin_op().await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                UserObject::all_users(),
                CoreUserAction::USER_CREATE,
            )
            .await?;

        let user = match self.repo.find_by_email_in_tx(db.tx(), &email).await {
            Err(e) if e.was_not_found() => {
                let new_user = NewUser::builder()
                    .email(&email)
                    .audit_info(audit_info.clone())
                    .build()
                    .expect("Could not build user");
                let mut user = self.repo.create_in_op(&mut db, new_user).await?;
                self.authz
                    .assign_role_to_subject(user.id, &RoleName::SUPERUSER)
                    .await?;
                let _ = user.assign_role(RoleName::SUPERUSER, audit_info);
                self.repo.update_in_op(&mut db, &mut user).await?;
                Some(user)
            }
            Err(e) => return Err(e),
            Ok(mut user) => {
                if user
                    .assign_role(RoleName::SUPERUSER, audit_info)
                    .did_execute()
                {
                    self.authz
                        .assign_role_to_subject(user.id, RoleName::SUPERUSER)
                        .await?;
                    self.repo.update_in_op(&mut db, &mut user).await?;
                    None
                } else {
                    return Ok(());
                }
            }
        };
        if let Some(user) = user {
            self.outbox
                .publish_persisted(db.tx(), CoreUserEvent::UserCreated { id: user.id, email })
                .await?;
        }
        db.commit().await?;
        Ok(())
    }
}
