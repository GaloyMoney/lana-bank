use async_graphql::*;

use admin_graphql_shared::primitives::*;

pub use lana_app::access::role::RolesByNameCursor;

use super::Role;

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
