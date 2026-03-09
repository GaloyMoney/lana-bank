use async_graphql::{connection::*, *};

use crate::{
    graphql::event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
    primitives::*,
};
use lana_app::access::permission_set::PermissionSet as DomainPermissionSet;
pub use lana_app::access::permission_set::PermissionSetsByIdCursor;

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
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn description(&self) -> &str {
        permission_sets_macro::find_by_name(&self.entity.name)
            .map(|e| e.description)
            .unwrap_or("")
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

impl From<DomainPermissionSet> for PermissionSet {
    fn from(permission_set: DomainPermissionSet) -> Self {
        Self {
            id: permission_set.id.to_global_id(),
            permission_set_id: UUID::from(permission_set.id),
            entity: Arc::new(permission_set),
        }
    }
}
