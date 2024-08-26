use async_graphql::*;

use crate::{
    app::LavaApp,
    authorization::ObjectPermission,
    primitives::{Role, UserId},
    server::shared_graphql::primitives::UUID,
};

#[derive(InputObject)]
pub struct UserCreateInput {
    pub email: String,
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct User {
    user_id: UUID,
    email: String,
    roles: Vec<Role>,
}

#[ComplexObject]
impl User {
    async fn user_permissions(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<ObjectPermission>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let permissions = app
            .users()
            .get_user_permissions(UserId::from(&self.user_id))
            .await?;
        Ok(permissions)
    }
}

#[derive(SimpleObject)]
pub struct UserCreatePayload {
    user: User,
}

impl From<crate::user::User> for User {
    fn from(user: crate::user::User) -> Self {
        Self {
            user_id: UUID::from(user.id),
            roles: user.current_roles().into_iter().map(Role::from).collect(),
            email: user.email,
        }
    }
}

impl From<crate::user::User> for UserCreatePayload {
    fn from(user: crate::user::User) -> Self {
        Self {
            user: User::from(user),
        }
    }
}

#[derive(InputObject)]
pub struct UserAssignRoleInput {
    pub id: UUID,
    pub role: Role,
}

#[derive(SimpleObject)]
pub struct UserAssignRolePayload {
    user: User,
}

impl From<crate::user::User> for UserAssignRolePayload {
    fn from(user: crate::user::User) -> Self {
        Self {
            user: User::from(user),
        }
    }
}

#[derive(InputObject)]
pub struct UserRevokeRoleInput {
    pub id: UUID,
    pub role: Role,
}

#[derive(SimpleObject)]
pub struct UserRevokeRolePayload {
    user: User,
}

impl From<crate::user::User> for UserRevokeRolePayload {
    fn from(user: crate::user::User) -> Self {
        Self {
            user: User::from(user),
        }
    }
}
