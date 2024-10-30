use async_graphql::{dataloader::DataLoader, *};

use crate::shared_graphql::{convert::ToGlobalId, primitives::UUID};
use lava_app::primitives::PolicyId;

use super::{committee::Committee, LavaDataLoader};

pub use governance::policy_cursor::PolicyByCreatedAtCursor;

#[derive(SimpleObject, Clone)]
pub struct Policy {
    id: ID,
    policy_id: UUID,
    approval_process_type: ApprovalProcessType,
    rules: ApprovalRules,
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(super) struct CommitteeThreshold {
    threshold: usize,
    #[graphql(skip)]
    committee_id: governance::CommitteeId,
}

#[ComplexObject]
impl CommitteeThreshold {
    async fn committee(&self, ctx: &Context<'_>) -> async_graphql::Result<Committee> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        let committee = loader
            .load_one(self.committee_id)
            .await?
            .expect("committee not found");
        Ok(committee)
    }
}

#[derive(SimpleObject, Clone)]
pub(super) struct SystemApproval {
    auto_approve: bool,
}

#[derive(async_graphql::Union, Clone)]
pub(super) enum ApprovalRules {
    CommitteeThreshold(CommitteeThreshold),
    System(SystemApproval),
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalProcessType {
    WithdrawApproval,
    CreditFacilityApproval,
    DisbursementApproval,
}

impl From<&governance::ApprovalProcessType> for ApprovalProcessType {
    fn from(process_type: &governance::ApprovalProcessType) -> Self {
        if process_type == &lava_app::governance::APPROVE_WITHDRAW_PROCESS {
            Self::WithdrawApproval
        } else if process_type == &lava_app::governance::APPROVE_CREDIT_FACILITY_PROCESS {
            Self::CreditFacilityApproval
        } else if process_type == &lava_app::governance::APPROVE_DISBURSEMENT_PROCESS {
            Self::DisbursementApproval
        } else {
            panic!("Unknown ApprovalProcessType")
        }
    }
}

#[derive(InputObject)]
pub struct PolicyAssignCommitteeInput {
    pub policy_id: UUID,
    pub committee_id: UUID,
    pub threshold: usize,
}

#[derive(SimpleObject)]
pub struct PolicyAssignCommitteePayload {
    policy: Policy,
}

impl ToGlobalId for PolicyId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("policy:{}", self))
    }
}

impl From<governance::Policy> for Policy {
    fn from(policy: governance::Policy) -> Self {
        Self {
            id: policy.id.to_global_id(),
            policy_id: policy.id.into(),
            approval_process_type: ApprovalProcessType::from(&policy.process_type),
            rules: ApprovalRules::from(policy.rules),
        }
    }
}

impl From<governance::Policy> for PolicyAssignCommitteePayload {
    fn from(policy: governance::Policy) -> Self {
        Self {
            policy: policy.into(),
        }
    }
}

impl From<governance::ApprovalRules> for ApprovalRules {
    fn from(rules: governance::ApprovalRules) -> Self {
        match rules {
            governance::ApprovalRules::CommitteeThreshold {
                threshold,
                committee_id,
            } => ApprovalRules::CommitteeThreshold(CommitteeThreshold {
                threshold,
                committee_id,
            }),
            governance::ApprovalRules::SystemAutoApprove => {
                ApprovalRules::System(SystemApproval { auto_approve: true })
            }
        }
    }
}
