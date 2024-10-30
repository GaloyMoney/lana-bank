use async_graphql::{dataloader::DataLoader, *};

use std::collections::HashSet;

use crate::shared_graphql::{
    convert::ToGlobalId,
    primitives::{Timestamp, UUID},
    withdraw::Withdrawal,
};
use lava_app::primitives::{ApprovalProcessId, CreditFacilityId, UserId, WithdrawId};

use super::{
    credit_facility::CreditFacility,
    policy::{ApprovalProcessType, ApprovalRules, Policy},
    user::User,
    LavaDataLoader,
};

pub use governance::{
    approval_process_cursor::ApprovalProcessByCreatedAtCursor, ApprovalProcessStatus,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ApprovalProcess {
    id: ID,
    approval_process_id: UUID,
    rules: ApprovalRules,
    approval_process_type: ApprovalProcessType,
    status: ApprovalProcessStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    committee_id: Option<governance::CommitteeId>,
    #[graphql(skip)]
    approvers: HashSet<UserId>,
    #[graphql(skip)]
    deniers: HashSet<UserId>,
    #[graphql(skip)]
    policy_id: governance::PolicyId,
    #[graphql(skip)]
    target_ref: String,
}

#[ComplexObject]
impl ApprovalProcess {
    async fn policy(&self, ctx: &Context<'_>) -> async_graphql::Result<Policy> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        let policy = loader
            .load_one(self.policy_id)
            .await?
            .expect("policy not found");
        Ok(policy)
    }

    async fn voters(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ApprovalProcessVoter>> {
        if let Some(committee_id) = self.committee_id {
            let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
            let committee = loader
                .load_one(committee_id)
                .await?
                .expect("committee not found");
            let mut approvers = self.approvers.clone();
            let mut deniers = self.deniers.clone();
            let mut voters: Vec<_> = committee
                .user_ids
                .iter()
                .map(|user_id| ApprovalProcessVoter {
                    user_id: *user_id,
                    still_eligible: true,
                    did_vote: approvers.contains(user_id) || deniers.contains(user_id),
                    did_approve: approvers.remove(user_id),
                    did_deny: deniers.remove(user_id),
                })
                .collect();
            voters.extend(
                approvers
                    .into_iter()
                    .map(|user_id| ApprovalProcessVoter {
                        user_id,
                        still_eligible: false,
                        did_vote: true,
                        did_approve: true,
                        did_deny: false,
                    })
                    .chain(deniers.into_iter().map(|user_id| ApprovalProcessVoter {
                        user_id,
                        still_eligible: false,
                        did_vote: true,
                        did_approve: false,
                        did_deny: true,
                    })),
            );
            Ok(voters)
        } else {
            Ok(vec![])
        }
    }

    async fn target(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcessTarget> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        match self.approval_process_type {
            ApprovalProcessType::WithdrawApproval => {
                let withdrawal = loader
                    .load_one(
                        self.target_ref
                            .parse::<WithdrawId>()
                            .expect("invalid target ref"),
                    )
                    .await?
                    .expect("withdrawal not found");
                Ok(ApprovalProcessTarget::Withdrawal(withdrawal))
            }
            ApprovalProcessType::CreditFacilityApproval => {
                let credit_facility = loader
                    .load_one(
                        self.target_ref
                            .parse::<CreditFacilityId>()
                            .expect("invalid target ref"),
                    )
                    .await?
                    .expect("credit facility not found");
                Ok(ApprovalProcessTarget::CreditFacility(credit_facility))
            }
        }
    }
}

impl ToGlobalId for ApprovalProcessId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("approval_process:{}", self))
    }
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ApprovalProcessVoter {
    #[graphql(skip)]
    user_id: UserId,
    still_eligible: bool,
    did_vote: bool,
    did_approve: bool,
    did_deny: bool,
}

#[ComplexObject]
impl ApprovalProcessVoter {
    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
        let users = loader
            .load_one(self.user_id)
            .await?
            .expect("user not found");

        Ok(users)
    }
}

impl From<governance::ApprovalProcess> for ApprovalProcess {
    fn from(process: governance::ApprovalProcess) -> Self {
        Self {
            id: process.id.to_global_id(),
            approval_process_id: process.id.into(),
            approval_process_type: ApprovalProcessType::from(&process.process_type),
            status: process.status(),
            created_at: process.created_at().into(),
            committee_id: process.committee_id(),
            approvers: process.approvers(),
            deniers: process.deniers(),
            policy_id: process.policy_id,
            target_ref: process.target_ref().to_string(),
            rules: process.rules.into(),
        }
    }
}

#[derive(async_graphql::Union, Clone)]
pub(super) enum ApprovalProcessTarget {
    Withdrawal(Withdrawal),
    CreditFacility(CreditFacility),
}

#[derive(InputObject)]
pub struct ApprovalProcessApproveInput {
    pub process_id: UUID,
}

#[derive(SimpleObject)]
pub struct ApprovalProcessApprovePayload {
    approval_process: ApprovalProcess,
}

impl From<governance::ApprovalProcess> for ApprovalProcessApprovePayload {
    fn from(process: governance::ApprovalProcess) -> Self {
        Self {
            approval_process: process.into(),
        }
    }
}

#[derive(InputObject)]
pub struct ApprovalProcessDenyInput {
    pub process_id: UUID,
}

#[derive(SimpleObject)]
pub struct ApprovalProcessDenyPayload {
    approval_process: ApprovalProcess,
}

impl From<governance::ApprovalProcess> for ApprovalProcessDenyPayload {
    fn from(process: governance::ApprovalProcess) -> Self {
        Self {
            approval_process: process.into(),
        }
    }
}
