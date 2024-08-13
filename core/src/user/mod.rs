mod config;
mod entity;
pub mod error;
mod repo;

use crate::{
    authorization::{Authorization, Object, UserAction},
    primitives::{Role, Subject, UserId},
};

pub use config::*;
pub use entity::*;
use error::UserError;
pub use repo::UserRepo;

#[derive(Clone)]
pub struct Users {
    pool: sqlx::PgPool,
    authz: Authorization,
    repo: UserRepo,
}

impl Users {
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Authorization,
        config: UserConfig,
    ) -> Result<Self, UserError> {
        let repo = UserRepo::new(pool);
        let users = Self {
            pool: pool.clone(),
            authz: authz.clone(),
            repo,
        };

        if let Some(email) = config.superuser_email {
            users.create_and_assign_role_to_superuser(email).await?;
        }

        Ok(users)
    }

    async fn create_and_assign_role_to_superuser(&self, email: String) -> Result<(), UserError> {
        if self.find_by_email(&email).await?.is_none() {
            let new_user = NewUser::builder()
                .email(&email)
                .build()
                .expect("Could not build user");
            let mut db = self.pool.begin().await?;
            let user = self.repo.create_in_tx(&mut db, new_user).await?;
            self.authz
                .assign_role_to_subject(user.id, &Role::Superuser)
                .await?;
            db.commit().await?;
        }
        Ok(())
    }

    pub fn repo(&self) -> &UserRepo {
        &self.repo
    }

    pub async fn create_user(
        &self,
        sub: &Subject,
        email: impl Into<String>,
    ) -> Result<User, UserError> {
        self.authz
            .check_permission(sub, Object::User, UserAction::Create)
            .await?;
        let new_user = NewUser::builder()
            .email(email)
            .build()
            .expect("Could not build user");
        let mut db = self.pool.begin().await?;
        let user = self.repo.create_in_tx(&mut db, new_user).await?;
        db.commit().await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, sub: &Subject, id: UserId) -> Result<Option<User>, UserError> {
        self.authz
            .check_permission(sub, Object::User, UserAction::Read)
            .await?;
        match self.repo.find_by_id(id).await {
            Ok(user) => Ok(Some(user)),
            Err(UserError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_email(&self, email: impl Into<String>) -> Result<Option<User>, UserError> {
        match self.repo.find_by_email(email).await {
            Ok(user) => Ok(Some(user)),
            Err(UserError::CouldNotFindByEmail(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_users(&self, sub: &Subject) -> Result<Vec<User>, UserError> {
        self.authz
            .check_permission(sub, Object::User, UserAction::List)
            .await?;
        self.repo.list().await
    }

    pub async fn assign_role_to_user(
        &self,
        sub: &Subject,
        id: UserId,
        role: Role,
    ) -> Result<User, UserError> {
        self.authz
            .check_permission(sub, Object::User, UserAction::AssignRole(role))
            .await?;

        let mut user = self.repo.find_by_id(id).await?;
        if user.assign_role(role) {
            self.authz.assign_role_to_subject(user.id, &role).await?;
            self.repo.persist(&mut user).await?;
        }

        Ok(user)
    }

    pub async fn revoke_role_from_user(
        &self,
        sub: &Subject,
        id: UserId,
        role: Role,
    ) -> Result<User, UserError> {
        self.authz
            .check_permission(sub, Object::User, UserAction::RevokeRole(role))
            .await?;

        let mut user = self.repo.find_by_id(id).await?;
        if user.revoke_role(role) {
            self.authz.revoke_role_from_subject(user.id, &role).await?;
            self.repo.persist(&mut user).await?;
        }

        Ok(user)
    }
}
