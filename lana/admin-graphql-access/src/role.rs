use async_graphql::*;

use admin_graphql_shared::primitives::*;
use lana_app::access::role::Role as DomainRole;

pub use lana_app::access::role::RolesByNameCursor;

use super::PermissionSet;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Role {
    id: ID,
    role_id: UUID,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainRole>,
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
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let ids: Vec<_> = self.entity.permission_sets.iter().copied().collect();
        let loaded: std::collections::HashMap<PermissionSetId, PermissionSet> =
            app.access().find_all_permission_sets(&ids).await?;
        Ok(loaded.into_values().collect())
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
mutation_payload! { RoleCreatePayload, role: Role }

#[derive(InputObject)]
pub struct RoleAddPermissionSetsInput {
    pub role_id: UUID,
    pub permission_set_ids: Vec<UUID>,
}
mutation_payload! { RoleAddPermissionSetsPayload, role: Role }

#[derive(InputObject)]
pub struct RoleRemovePermissionSetsInput {
    pub role_id: UUID,
    pub permission_set_ids: Vec<UUID>,
}
mutation_payload! { RoleRemovePermissionSetsPayload, role: Role }
