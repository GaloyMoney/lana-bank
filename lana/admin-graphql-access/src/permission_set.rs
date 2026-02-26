use async_graphql::*;

use admin_graphql_shared::primitives::*;
use lana_app::access::permission_set::PermissionSet as DomainPermissionSet;

pub use lana_app::access::permission_set::PermissionSetsByIdCursor;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PermissionSet {
    id: ID,
    permission_set_id: UUID,

    #[graphql(skip)]
    pub entity: Arc<DomainPermissionSet>,
}

#[ComplexObject]
impl PermissionSet {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn description(&self) -> &str {
        permission_sets_macro::find_by_name(&self.entity.name)
            .map(|e| e.description)
            .unwrap_or("")
    }
}

impl From<DomainPermissionSet> for PermissionSet {
    fn from(permission_set: DomainPermissionSet) -> Self {
        Self {
            id: permission_set.id.to_global_id(),
            permission_set_id: UUID::from(permission_set.id),
            entity: Arc::new(permission_set),
        }
    }
}
