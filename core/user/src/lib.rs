#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use std::collections::HashMap;

use audit::AuditSvc;
use authz::{Authorization, PermissionCheck};
use outbox::{Outbox, OutboxEventMarker};

use entity::*;
use error::*;
pub use primitives::*;
use repo::*;

pub struct Users<Audit, E>
where
    Audit: AuditSvc,
    E: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static + Unpin,
{
    pool: sqlx::PgPool,
    authz: Authorization<Audit, Role>,
    outbox: Outbox<E>,
    repo: UserRepo,
}
impl<Audit, E> Users<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<UserModuleAction>,
    <Audit as AuditSvc>::Object: From<UserObject>,
    E: OutboxEventMarker<UserModuleEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, Role>,
        outbox: &Outbox<E>,
        superuser_email: Option<String>,
    ) -> Result<Self, UserError> {
        let repo = UserRepo::new(pool);
        let users = Self {
            pool: pool.clone(),
            repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
        };

        if let Some(email) = superuser_email {
            users.create_and_assign_role_to_superuser(email).await?;
        }

        Ok(users)
    }

    pub async fn can_create_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                UserObject::User(UserAllOrOne::All),
                UserModuleAction::User(UserEntityAction::Create),
                enforce,
            )
            .await?)
    }

    pub async fn create_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        email: impl Into<String>,
    ) -> Result<User, UserError> {
        let audit_info = self
            .can_create_user(sub, true)
            .await?
            .expect("audit info missing");

        let new_user = NewUser::builder()
            .email(email)
            .audit_info(audit_info)
            .build()
            .expect("Could not build user");
        let mut db = self.pool.begin().await?;
        let user = self.repo.create_in_tx(&mut db, new_user).await?;
        self.outbox
            .persist(&mut db, UserModuleEvent::UserCreated { id: user.id })
            .await?;
        db.commit().await?;
        Ok(user)
    }

    pub async fn find_by_id(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        id: UserId,
    ) -> Result<Option<User>, UserError> {
        self.authz
            .enforce_permission(
                sub,
                UserObject::User(UserAllOrOne::ById(id)),
                UserModuleAction::User(UserEntityAction::Read),
            )
            .await?;
        match self.repo.find_by_id(id).await {
            Ok(user) => Ok(Some(user)),
            Err(UserError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_all<T: From<User>>(
        &self,
        ids: &[UserId],
    ) -> Result<HashMap<UserId, T>, UserError> {
        self.repo.find_all(ids).await
    }

    pub async fn list_users(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
    ) -> Result<Vec<User>, UserError> {
        self.authz
            .enforce_permission(
                sub,
                UserObject::User(UserAllOrOne::All),
                UserModuleAction::User(UserEntityAction::List),
            )
            .await?;

        Ok(self.repo.list_by_email(Default::default()).await?.entities)
    }

    pub async fn can_assign_role_to_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: UserId,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                UserObject::User(UserAllOrOne::ById(user_id)),
                UserModuleAction::User(UserEntityAction::AssignRole),
                enforce,
            )
            .await?)
    }

    pub async fn assign_role_to_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        id: UserId,
        role: Role,
    ) -> Result<User, UserError> {
        if role == Role::SUPERUSER {
            return Err(UserError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }
        let audit_info = self
            .can_assign_role_to_user(sub, id, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        if user.assign_role(role.clone(), audit_info) {
            self.authz.assign_role_to_subject(user.id, role).await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }

    pub async fn can_revoke_role_from_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        user_id: UserId,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                UserObject::User(UserAllOrOne::ById(user_id)),
                UserModuleAction::User(UserEntityAction::RevokeRole),
                enforce,
            )
            .await?)
    }

    pub async fn revoke_role_from_user(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        id: UserId,
        role: Role,
    ) -> Result<User, UserError> {
        if role == Role::SUPERUSER {
            return Err(UserError::AuthorizationError(
                authz::error::AuthorizationError::NotAuthorized,
            ));
        }
        let audit_role = self
            .can_revoke_role_from_user(sub, id, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        if user.revoke_role(role.clone(), audit_role) {
            self.authz.revoke_role_from_subject(user.id, role).await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }

    async fn create_and_assign_role_to_superuser(&self, email: String) -> Result<(), UserError> {
        let mut db = self.pool.begin().await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                &mut db,
                UserObject::User(UserAllOrOne::All),
                UserModuleAction::User(UserEntityAction::Create),
            )
            .await?;

        let user = match self.repo.find_by_email_in_tx(&mut db, &email).await {
            Err(UserError::NotFound) => {
                let new_user = NewUser::builder()
                    .email(&email)
                    .audit_info(audit_info.clone())
                    .build()
                    .expect("Could not build user");
                let mut user = self.repo.create_in_tx(&mut db, new_user).await?;
                self.authz
                    .assign_role_to_subject(user.id, &Role::SUPERUSER)
                    .await?;
                user.assign_role(Role::SUPERUSER, audit_info);
                self.repo.update_in_tx(&mut db, &mut user).await?;
                Some(user)
            }
            Err(e) => return Err(e),
            Ok(mut user) => {
                if user.assign_role(Role::SUPERUSER, audit_info) {
                    self.authz
                        .assign_role_to_subject(user.id, Role::SUPERUSER)
                        .await?;
                    self.repo.update_in_tx(&mut db, &mut user).await?;
                    None
                } else {
                    return Ok(());
                }
            }
        };
        if let Some(user) = user {
            self.outbox
                .persist(&mut db, UserModuleEvent::UserCreated { id: user.id })
                .await?;
        }
        db.commit().await?;
        Ok(())
    }
}
