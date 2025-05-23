use async_graphql::*;

use crate::graphql::access::PermissionSet;
use crate::graphql::loader::LanaDataLoader;
use crate::primitives::*;
use lana_app::access::role::Role as DomainRole;
pub use lana_app::access::role::RolesByNameCursor;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
#[graphql(name = "RoleEntity")]
pub struct Role {
    id: ID,
    role_id: UUID,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainRole>,
}

#[ComplexObject]
impl Role {
    async fn name(&self) -> &str {
        self.entity.name.name()
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
}

impl From<DomainRole> for Role {
    fn from(role: DomainRole) -> Self {
        Self {
            id: role.id.to_global_id(),
            role_id: UUID::from(role.id),
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
pub struct RoleRemovePermissionSetInput {
    pub role_id: UUID,
    pub permission_set_id: UUID,
}
crate::mutation_payload! { RoleRemovePermissionSetPayload, role: Role }
