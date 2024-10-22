mod config;
mod entity;
pub mod error;
mod repo;

use std::collections::HashMap;

use crate::{
    audit::Audit,
    authorization::{error::AuthorizationError, Authorization, Object, UserAction},
    data_export::Export,
    primitives::{AuditInfo, Role, Subject, SystemNode, UserId},
};

pub use config::*;
pub use entity::*;
use error::UserError;
pub use repo::UserRepo;

#[derive(Clone)]
pub struct Users {
    pool: sqlx::PgPool,
    authz: Authorization,
    audit: Audit,
    repo: UserRepo,
}

impl Users {
    pub async fn init(
        pool: &sqlx::PgPool,
        config: UserConfig,
        authz: &Authorization,
        audit: &Audit,
        export: &Export,
    ) -> Result<Self, UserError> {
        let repo = UserRepo::new(pool, export);
        let users = Self {
            pool: pool.clone(),
            authz: authz.clone(),
            audit: audit.clone(),
            repo,
        };

        if let Some(email) = config.superuser_email {
            users.create_and_assign_role_to_superuser(email).await?;
        }

        Ok(users)
    }

    async fn create_and_assign_role_to_superuser(&self, email: String) -> Result<(), UserError> {
        let subject = Subject::System(SystemNode::Init);
        let audit_info = self
            .audit
            .record_entry(&subject, Object::User, UserAction::Create, true)
            .await?;

        if self.find_by_email(&email).await?.is_none() {
            let new_user = NewUser::builder()
                .email(&email)
                .audit_info(audit_info)
                .build()
                .expect("Could not build user");
            let mut db = self.pool.begin().await?;
            let mut user = self.repo.create_in_tx(&mut db, new_user).await?;
            user.assign_role(Role::Superuser, audit_info);
            self.authz
                .assign_role_to_subject(user.id, &Role::Superuser)
                .await?;
            self.repo.update_in_tx(&mut db, &mut user).await?;
            db.commit().await?;
        }
        Ok(())
    }

    pub fn repo(&self) -> &UserRepo {
        &self.repo
    }

    pub async fn can_create_user(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(sub, Object::User, UserAction::Create, enforce)
            .await?)
    }

    pub async fn create_user(
        &self,
        sub: &Subject,
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
        db.commit().await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, sub: &Subject, id: UserId) -> Result<Option<User>, UserError> {
        self.authz
            .enforce_permission(sub, Object::User, UserAction::Read)
            .await?;
        match self.repo.find_by_id(id).await {
            Ok(user) => Ok(Some(user)),
            Err(UserError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_id_internal(&self, id: UserId) -> Result<Option<User>, UserError> {
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

    pub async fn find_by_email(&self, email: impl Into<String>) -> Result<Option<User>, UserError> {
        match self.repo.find_by_email(email.into()).await {
            Ok(user) => Ok(Some(user)),
            Err(UserError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_users(&self, sub: &Subject) -> Result<Vec<User>, UserError> {
        self.authz
            .enforce_permission(sub, Object::User, UserAction::List)
            .await?;

        Ok(self.repo.list_by_email(Default::default()).await?.entities)
    }

    pub async fn can_assign_role_to_user(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(sub, Object::User, UserAction::AssignRole, enforce)
            .await?)
    }

    pub async fn assign_role_to_user(
        &self,
        sub: &Subject,
        id: UserId,
        role: Role,
    ) -> Result<User, UserError> {
        if role == Role::Superuser {
            return Err(UserError::AuthorizationError(
                AuthorizationError::NotAuthorized,
            ));
        }
        let audit_info = self
            .can_assign_role_to_user(sub, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        if user.assign_role(role, audit_info) {
            self.authz.assign_role_to_subject(user.id, &role).await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }

    pub async fn can_revoke_role_from_user(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, UserError> {
        Ok(self
            .authz
            .evaluate_permission(sub, Object::User, UserAction::RevokeRole, enforce)
            .await?)
    }

    pub async fn revoke_role_from_user(
        &self,
        sub: &Subject,
        id: UserId,
        role: Role,
    ) -> Result<User, UserError> {
        if role == Role::Superuser {
            return Err(UserError::AuthorizationError(
                AuthorizationError::NotAuthorized,
            ));
        }
        let audit_role = self
            .can_revoke_role_from_user(sub, true)
            .await?
            .expect("audit info missing");

        let mut user = self.repo.find_by_id(id).await?;
        if user.revoke_role(role, audit_role) {
            self.authz.revoke_role_from_subject(user.id, &role).await?;
            self.repo.update(&mut user).await?;
        }

        Ok(user)
    }
}
