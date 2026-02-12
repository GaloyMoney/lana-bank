use async_graphql::*;

use crate::primitives::*;

use super::{approval_process::*, approval_rules::*};

pub(super) use lana_app::governance::{
    Policy as DomainPolicy, policy_cursor::PoliciesByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(super) struct Policy {
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
}

#[derive(InputObject)]
pub(super) struct PolicyAssignCommitteeInput {
    pub policy_id: UUID,
    pub committee_id: UUID,
    pub threshold: usize,
}

mutation_payload! { PolicyAssignCommitteePayload, policy: Policy }
