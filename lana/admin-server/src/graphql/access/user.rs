use async_graphql::*;

use crate::{graphql::loader::LanaDataLoader, primitives::*};
use lana_app::access::user::User as DomainUser;

use super::Role;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct User {
    id: ID,
    user_id: UUID,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainUser>,
}

impl From<DomainUser> for User {
    fn from(user: DomainUser) -> Self {
        Self {
            id: user.id.to_global_id(),
            user_id: UUID::from(user.id),
            created_at: user.created_at().into(),
            entity: Arc::new(user),
        }
    }
}

impl From<Arc<DomainUser>> for User {
    fn from(user: Arc<DomainUser>) -> Self {
        Self {
            id: user.id.to_global_id(),
            user_id: UUID::from(user.id),
            created_at: user.created_at().into(),
            entity: user,
        }
    }
}

#[ComplexObject]
impl User {
    async fn role(&self, ctx: &Context<'_>) -> async_graphql::Result<Role> {
        let role_id = self.entity.current_role();
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let role = loader.load_one(role_id).await?;
        role.ok_or_else(|| {
            Error::new(format!(
                "Data integrity error: Role with ID {} not found for user {}. This should never happen.",
                role_id, self.entity.id
            ))
        })
    }

    async fn email(&self) -> &str {
        &self.entity.email
    }

    async fn user_can_update_role_of_user(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .access()
            .users()
            .user_can_update_role_of_user(sub, None, false)
            .await
            .is_ok())
    }
}

#[derive(InputObject)]
pub struct UserCreateInput {
    pub email: String,
    pub role_id: UUID,
}

mutation_payload! { UserCreatePayload, user: User }

#[derive(InputObject)]
pub struct UserUpdateRoleInput {
    pub id: UUID,
    pub role_id: UUID,
}
mutation_payload! { UserUpdateRolePayload, user: User }
