use async_graphql::{connection::*, *};

use crate::primitives::*;

use super::{
    approval_process::*,
    approval_rules::*,
    event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
};

pub use lana_app::governance::{Policy as DomainPolicy, policy_cursor::PoliciesByCreatedAtCursor};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Policy {
    id: ID,
    policy_id: UUID,
    approval_process_type: ApprovalProcessType,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainPolicy>,
}

impl From<DomainPolicy> for Policy {
    fn from(policy: DomainPolicy) -> Self {
        Self {
            id: policy.id.to_global_id(),
            policy_id: policy.id.into(),
            approval_process_type: ApprovalProcessType::from(&policy.process_type),
            entity: Arc::new(policy),
        }
    }
}

#[ComplexObject]
impl Policy {
    async fn rules(&self) -> ApprovalRules {
        ApprovalRules::from(self.entity.rules)
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
pub struct PolicyAssignCommitteeInput {
    pub policy_id: UUID,
    pub committee_id: UUID,
}

mutation_payload! { PolicyAssignCommitteePayload, policy: Policy }
