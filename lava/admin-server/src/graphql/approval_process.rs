use async_graphql::*;

use lava_app::app::LavaApp;

use crate::primitives::*;

use super::{approval_rules::*, loader::LavaDataLoader, policy::*, user::User};

pub use governance::{
    approval_process_cursor::ApprovalProcessByCreatedAtCursor,
    ApprovalProcess as DomainApprovalProcess, ApprovalProcessStatus,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ApprovalProcess {
    id: ID,
    approval_process_id: UUID,
    approval_process_type: ApprovalProcessType,
    status: ApprovalProcessStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainApprovalProcess>,
}

impl From<DomainApprovalProcess> for ApprovalProcess {
    fn from(process: DomainApprovalProcess) -> Self {
        Self {
            id: process.id.to_global_id(),
            approval_process_id: process.id.into(),
            approval_process_type: ApprovalProcessType::from(&process.process_type),
            status: process.status(),
            created_at: process.created_at().into(),
            entity: Arc::new(process),
        }
    }
}

#[ComplexObject]
impl ApprovalProcess {
    async fn rules(&self) -> ApprovalRules {
        ApprovalRules::from(self.entity.rules)
    }

    async fn policy(&self, ctx: &Context<'_>) -> async_graphql::Result<Policy> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let policy = loader
            .load_one(self.entity.policy_id)
            .await?
            .expect("policy not found");
        Ok(policy)
    }

    async fn user_can_vote(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let committee = if let Some(committee_id) = self.entity.committee_id() {
            let loader = ctx.data_unchecked::<LavaDataLoader>();
            let committee = loader
                .load_one(committee_id)
                .await?
                .expect("committee not found");
            Some(committee.entity)
        } else {
            None
        };

        Ok(app
            .governance()
            .subject_can_vote(sub, &self.entity, committee.as_ref().map(AsRef::as_ref))
            .await?)
    }

    async fn voters(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ApprovalProcessVoter>> {
        if let Some(committee_id) = self.entity.committee_id() {
            let loader = ctx.data_unchecked::<LavaDataLoader>();
            let committee = loader
                .load_one(committee_id)
                .await?
                .expect("committee not found");
            let mut approvers = self.entity.approvers();
            let mut deniers = self.entity.deniers();
            let mut voters: Vec<_> = committee
                .entity
                .members()
                .into_iter()
                .map(|user_id| ApprovalProcessVoter {
                    still_eligible: true,
                    did_vote: approvers.contains(&user_id) || deniers.contains(&user_id),
                    did_approve: approvers.remove(&user_id),
                    did_deny: deniers.remove(&user_id),
                    user_id,
                    voted_at: self.entity.user_voted_at(user_id).map(Into::into),
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
                        voted_at: self.entity.user_voted_at(user_id).map(Into::into),
                    })
                    .chain(deniers.into_iter().map(|user_id| ApprovalProcessVoter {
                        user_id,
                        still_eligible: false,
                        did_vote: true,
                        did_approve: false,
                        did_deny: true,
                        voted_at: self.entity.user_voted_at(user_id).map(Into::into),
                    })),
            );
            Ok(voters)
        } else {
            Ok(vec![])
        }
    }

    // async fn target(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcessTarget> {
    //     let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();
    //     match self.approval_process_type {
    //         ApprovalProcessType::WithdrawApproval => {
    //             let withdrawal = loader
    //                 .load_one(
    //                     self.entity
    //                         .target_ref()
    //                         .parse::<WithdrawId>()
    //                         .expect("invalid target ref"),
    //                 )
    //                 .await?
    //                 .expect("withdrawal not found");
    //             Ok(ApprovalProcessTarget::Withdrawal(withdrawal))
    //         }
    //         ApprovalProcessType::CreditFacilityApproval => {
    //             let credit_facility = loader
    //                 .load_one(
    //                     self.entity
    //                         .target_ref()
    //                         .parse::<CreditFacilityId>()
    //                         .expect("invalid target ref"),
    //                 )
    //                 .await?
    //                 .expect("credit facility not found");
    //             Ok(ApprovalProcessTarget::CreditFacility(credit_facility))
    //         }
    //     }
    // }
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalProcessType {
    WithdrawApproval,
    CreditFacilityApproval,
}

impl From<&governance::ApprovalProcessType> for ApprovalProcessType {
    fn from(process_type: &governance::ApprovalProcessType) -> Self {
        if process_type == &lava_app::governance::APPROVE_WITHDRAW_PROCESS {
            Self::WithdrawApproval
        } else if process_type == &lava_app::governance::APPROVE_CREDIT_FACILITY_PROCESS {
            Self::CreditFacilityApproval
        } else {
            panic!("Unknown ApprovalProcessType")
        }
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
    voted_at: Option<Timestamp>,
}

#[ComplexObject]
impl ApprovalProcessVoter {
    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let users = loader
            .load_one(self.user_id)
            .await?
            .expect("user not found");

        Ok(users)
    }
}

// #[allow(clippy::large_enum_variant)]
// #[derive(async_graphql::Union, Clone)]
// pub(super) enum ApprovalProcessTarget {
//     Withdrawal(u32),
//     CreditFacility(String),
// }

#[derive(InputObject)]
pub struct ApprovalProcessApproveInput {
    pub process_id: UUID,
}
crate::mutation_payload! { ApprovalProcessApprovePayload, approval_process: ApprovalProcess }

#[derive(InputObject)]
pub struct ApprovalProcessDenyInput {
    pub process_id: UUID,
}
crate::mutation_payload! { ApprovalProcessDenyPayload, approval_process: ApprovalProcess }
