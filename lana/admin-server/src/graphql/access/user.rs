use async_graphql::{
    connection::{Connection, EmptyFields},
    *,
};
use es_entity::Sort;

use crate::{graphql::loader::LanaDataLoader, primitives::*};
use lana_app::access::user::User as DomainUser;
use lana_app::access::user::UsersSortBy as DomainUsersSortBy;

use super::Role;
use crate::graphql::event_timeline::{self, EventTimelineCursor, EventTimelineEntry};
use crate::graphql::primitives::SortDirection;

#[derive(SimpleObject, Clone)]
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("userId".to_string())
)]
pub struct User {
    user_id: UserId,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainUser>,
}

impl From<DomainUser> for User {
    fn from(user: DomainUser) -> Self {
        Self {
            user_id: user.id,
            created_at: user.created_at().into(),
            entity: Arc::new(user),
        }
    }
}

impl From<Arc<DomainUser>> for User {
    fn from(user: Arc<DomainUser>) -> Self {
        Self {
            user_id: user.id,
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

    async fn user_can_update_role_of_user(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .access()
            .users()
            .subject_can_update_role_of_user(sub, None, false)
            .await
            .is_ok())
    }

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        use es_entity::EsEntity as _;
        event_timeline::events_to_connection(self.entity.events(), first, after)
    }
}

#[derive(InputObject)]
pub struct UserCreateInput {
    pub email: String,
    pub role_id: RoleId,
}

crate::mutation_payload! { UserCreatePayload, user: User }

#[derive(InputObject)]
pub struct UserRoleUpdateInput {
    pub user_id: UserId,
    pub role_id: RoleId,
}
crate::mutation_payload! { UserRoleUpdatePayload, user: User }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UsersSortBy {
    #[default]
    CreatedAt,
    Email,
}

impl From<UsersSortBy> for DomainUsersSortBy {
    fn from(by: UsersSortBy) -> Self {
        match by {
            UsersSortBy::CreatedAt => DomainUsersSortBy::CreatedAt,
            UsersSortBy::Email => DomainUsersSortBy::Email,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct UsersSort {
    #[graphql(default)]
    pub by: UsersSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<UsersSort> for Sort<DomainUsersSortBy> {
    fn from(sort: UsersSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<UsersSort> for DomainUsersSortBy {
    fn from(sort: UsersSort) -> Self {
        sort.by.into()
    }
}
