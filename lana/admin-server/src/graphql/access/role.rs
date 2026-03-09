use async_graphql::{
    connection::{Connection, EmptyFields},
    *,
};
use es_entity::Sort;

use crate::graphql::access::PermissionSet;
use crate::graphql::event_timeline::{self, EventTimelineCursor, EventTimelineEntry};
use crate::graphql::loader::LanaDataLoader;
use crate::graphql::primitives::SortDirection;
use crate::primitives::*;
use lana_app::access::role::Role as DomainRole;
use lana_app::access::role::RolesSortBy as DomainRolesSortBy;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Role {
    id: ID,
    role_id: UUID,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainRole>,
}

#[ComplexObject]
impl Role {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn permission_sets(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<PermissionSet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let loaded = loader
            .load_many(self.entity.permission_sets.iter().copied())
            .await?;
        Ok(loaded.into_values().collect())
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

impl From<DomainRole> for Role {
    fn from(role: DomainRole) -> Self {
        Self {
            id: role.id.to_global_id(),
            role_id: UUID::from(role.id),
            created_at: role.created_at().into(),

            entity: Arc::new(role),
        }
    }
}

#[derive(InputObject)]
pub struct RoleCreateInput {
    pub name: String,
    pub permission_set_ids: Vec<UUID>,
}
crate::mutation_payload! { RoleCreatePayload, role: Role }

#[derive(InputObject)]
pub struct RoleAddPermissionSetsInput {
    pub role_id: UUID,
    pub permission_set_ids: Vec<UUID>,
}
crate::mutation_payload! { RoleAddPermissionSetsPayload, role: Role }

#[derive(InputObject)]
pub struct RoleRemovePermissionSetsInput {
    pub role_id: UUID,
    pub permission_set_ids: Vec<UUID>,
}
crate::mutation_payload! { RoleRemovePermissionSetsPayload, role: Role }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RolesSortBy {
    #[default]
    CreatedAt,
    Name,
}

impl From<RolesSortBy> for DomainRolesSortBy {
    fn from(by: RolesSortBy) -> Self {
        match by {
            RolesSortBy::CreatedAt => DomainRolesSortBy::CreatedAt,
            RolesSortBy::Name => DomainRolesSortBy::Name,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct RolesSort {
    #[graphql(default)]
    pub by: RolesSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<RolesSort> for Sort<DomainRolesSortBy> {
    fn from(sort: RolesSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<RolesSort> for DomainRolesSortBy {
    fn from(sort: RolesSort) -> Self {
        sort.by.into()
    }
}
