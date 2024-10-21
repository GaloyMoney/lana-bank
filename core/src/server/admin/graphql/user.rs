use async_graphql::{types::connection::*, *};
use serde::{Deserialize, Serialize};

use crate::{
    app::LavaApp,
    authorization::VisibleNavigationItems,
    primitives::{Role, Subject, UserId},
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
    async fn visible_navigation_items(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<VisibleNavigationItems> {
        let app = ctx.data_unchecked::<LavaApp>();
        let sub = Subject::User(UserId::from(&self.user_id));
        let permissions = app.authz().get_visible_navigation_items(&sub).await?;
        Ok(permissions)
    }

    async fn can_create_customer(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let sub = Subject::User(UserId::from(&self.user_id));
        Ok(app
            .customers()
            .user_can_create_customer(&sub, false)
            .await
            .is_ok())
    }

    async fn can_create_user(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let sub = Subject::User(UserId::from(&self.user_id));
        Ok(app.users().can_create_user(&sub, false).await.is_ok())
    }

    async fn can_assign_role_to_user(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let sub = Subject::User(UserId::from(&self.user_id));
        Ok(app
            .users()
            .can_assign_role_to_user(&sub, false)
            .await
            .is_ok())
    }

    async fn can_revoke_role_from_user(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let sub = Subject::User(UserId::from(&self.user_id));
        Ok(app
            .users()
            .can_revoke_role_from_user(&sub, false)
            .await
            .is_ok())
    }

    async fn can_create_terms_template(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let sub = Subject::User(UserId::from(&self.user_id));
        Ok(app
            .terms_templates()
            .user_can_create_terms_template(&sub, false)
            .await
            .is_ok())
    }

    async fn can_update_terms_template(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let sub = Subject::User(UserId::from(&self.user_id));
        Ok(app
            .terms_templates()
            .user_can_update_terms_template(&sub, false)
            .await
            .is_ok())
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

#[derive(Serialize, Deserialize)]
pub(super) struct UserByEmailCursor {
    pub email: String,
    pub id: UserId,
}

impl CursorType for UserByEmailCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(&self).expect("could not serialize token");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

impl From<(UserId, &str)> for UserByEmailCursor {
    fn from((id, email): (UserId, &str)) -> Self {
        Self {
            id,
            email: email.to_string(),
        }
    }
}

impl From<UserByEmailCursor> for crate::user::UserByEmailCursor {
    fn from(cursor: UserByEmailCursor) -> Self {
        Self {
            id: cursor.id,
            email: cursor.email,
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
