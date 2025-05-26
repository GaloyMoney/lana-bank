use async_graphql::*;

use crate::primitives::*;
use lana_app::access::user::User as DomainUser;

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
    async fn role(&self) -> Option<UUID> {
        self.entity.current_role().map(UUID::from)
    }

    async fn email(&self) -> &str {
        &self.entity.email
    }

    async fn subject_can_assign_role_to_user(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .access()
            .users()
            .subject_can_assign_role_to_user(sub, None, false)
            .await
            .is_ok())
    }

    async fn subject_can_revoke_role_from_user(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .access()
            .users()
            .subject_can_revoke_role_from_user(sub, None, false)
            .await
            .is_ok())
    }
}

#[derive(InputObject)]
pub struct UserCreateInput {
    pub email: String,
}

mutation_payload! { UserCreatePayload, user: User }

#[derive(InputObject)]
pub struct UserAssignRoleInput {
    pub id: UUID,
    pub role_id: UUID,
}
mutation_payload! { UserAssignRolePayload, user: User }

#[derive(InputObject)]
pub struct UserRevokeRoleInput {
    pub id: UUID,
    pub role_id: UUID,
}

mutation_payload! { UserRevokeRolePayload, user: User }
