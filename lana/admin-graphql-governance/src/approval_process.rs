use async_graphql::*;

use std::sync::Arc;

use admin_graphql_access::User;
use admin_graphql_shared::primitives::*;

use super::approval_rules::ApprovalRules;
use super::policy::Policy;

pub use lana_app::governance::{
    ApprovalProcess as DomainApprovalProcess, ApprovalProcessStatus,
    ApprovalProcessType as DomainApprovalProcessType, Committee as DomainCommittee,
    approval_process_cursor::ApprovalProcessesByCreatedAtCursor,
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
    pub entity: Arc<DomainApprovalProcess>,
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

    async fn denied_reason(&self) -> Option<&str> {
        self.entity.denied_reason()
    }

    async fn policy(&self, ctx: &Context<'_>) -> async_graphql::Result<Policy> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let policy = app
            .governance()
            .find_policy(sub, self.entity.policy_id)
            .await?
            .expect("policy not found");
        Ok(Policy::from(policy))
    }

    async fn user_can_submit_decision(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let committee = if let Some(committee_id) = self.entity.committee_id() {
            let committee = app
                .governance()
                .find_committee_by_id(sub, committee_id)
                .await?
                .expect("committee not found");
            Some(committee)
        } else {
            None
        };

        Ok(app
            .governance()
            .subject_can_submit_decision(sub, &self.entity, committee.as_ref())
            .await?)
    }

    async fn voters(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ApprovalProcessVoter>> {
        if let Some(committee_id) = self.entity.committee_id() {
            let (app, _sub) = app_and_sub_from_ctx!(ctx);
            let committee: DomainCommittee = app
                .governance()
                .find_all_committees(&[committee_id])
                .await?
                .into_values()
                .next()
                .expect("committee not found");

            let mut approvers = self.entity.approvers();
            let mut deniers = self.entity.deniers();
            let mut voters: Vec<_> = committee
                .members()
                .into_iter()
                .map(|member_id| ApprovalProcessVoter {
                    still_eligible: true,
                    did_vote: approvers.contains(&member_id) || deniers.contains(&member_id),
                    did_approve: approvers.remove(&member_id),
                    did_deny: deniers.remove(&member_id),
                    user_id: UserId::from(member_id),
                    voted_at: self.entity.member_voted_at(member_id).map(Into::into),
                })
                .collect();
            voters.extend(
                approvers
                    .into_iter()
                    .map(|member_id| ApprovalProcessVoter {
                        user_id: UserId::from(member_id),
                        still_eligible: false,
                        did_vote: true,
                        did_approve: true,
                        did_deny: false,
                        voted_at: self.entity.member_voted_at(member_id).map(Into::into),
                    })
                    .chain(deniers.into_iter().map(|member_id| ApprovalProcessVoter {
                        user_id: UserId::from(member_id),
                        still_eligible: false,
                        did_vote: true,
                        did_approve: false,
                        did_deny: true,
                        voted_at: self.entity.member_voted_at(member_id).map(Into::into),
                    })),
            );
            Ok(voters)
        } else {
            Ok(vec![])
        }
    }
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum ApprovalProcessType {
    WithdrawalApproval,
    DisbursalApproval,
    CreditFacilityProposalApproval,
}

impl From<&DomainApprovalProcessType> for ApprovalProcessType {
    fn from(process_type: &DomainApprovalProcessType) -> Self {
        if process_type == &lana_app::governance::APPROVE_WITHDRAWAL_PROCESS {
            Self::WithdrawalApproval
        } else if process_type == &lana_app::governance::APPROVE_DISBURSAL_PROCESS {
            Self::DisbursalApproval
        } else if process_type == &lana_app::governance::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS {
            Self::CreditFacilityProposalApproval
        } else {
            panic!("Unknown approval process type: {process_type:?}");
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
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let user = app
            .access()
            .users()
            .find_by_id(sub, self.user_id)
            .await?
            .expect("user not found");
        Ok(User::from(user))
    }
}

#[derive(InputObject)]
pub struct ApprovalProcessApproveInput {
    pub process_id: UUID,
}
mutation_payload! { ApprovalProcessApprovePayload, approval_process: ApprovalProcess }

#[derive(InputObject)]
pub struct ApprovalProcessDenyInput {
    pub process_id: UUID,
}
mutation_payload! { ApprovalProcessDenyPayload, approval_process: ApprovalProcess }
