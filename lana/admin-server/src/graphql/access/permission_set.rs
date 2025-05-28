use async_graphql::*;

use crate::primitives::*;
pub use lana_app::access::permission_set::PermissionSetsByIdCursor;
use lana_app::{
    access::permission_set::PermissionSet as DomainPermissionSet, rbac::PermissionSetName,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PermissionSet {
    id: ID,
    permission_set_id: UUID,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainPermissionSet>,
}

#[ComplexObject]
impl PermissionSet {
    async fn name(&self) -> PermissionSetName {
        self.entity
            .name
            .parse()
            .expect("Invalid permission set name")
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
