use async_graphql::*;

use crate::access::User;
use crate::primitives::*;

pub use lana_app::governance::{
    ApprovalProcess as DomainApprovalProcess, ApprovalProcessStatus,
    ApprovalProcessType as DomainApprovalProcessType, ApprovalRules as DomainApprovalRules,
    Committee as DomainCommittee, CommitteeId, Policy as DomainPolicy,
    approval_process_cursor::ApprovalProcessesByCreatedAtCursor,
    committee_cursor::CommitteesByCreatedAtCursor, policy_cursor::PoliciesByCreatedAtCursor,
};

// ── ApprovalProcess ────────────────────────────────────────────────────

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

// ── ApprovalRules ──────────────────────────────────────────────────────

#[derive(async_graphql::Union)]
pub enum ApprovalRules {
    System(SystemApproval),
    CommitteeThreshold(CommitteeThreshold),
}

impl From<DomainApprovalRules> for ApprovalRules {
    fn from(rules: DomainApprovalRules) -> Self {
        match rules {
            DomainApprovalRules::CommitteeThreshold {
                threshold,
                committee_id,
            } => ApprovalRules::CommitteeThreshold(CommitteeThreshold {
                threshold,
                committee_id,
            }),
            DomainApprovalRules::SystemAutoApprove => {
                ApprovalRules::System(SystemApproval { auto_approve: true })
            }
        }
    }
}

#[derive(SimpleObject)]
pub struct SystemApproval {
    auto_approve: bool,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CommitteeThreshold {
    threshold: usize,
    #[graphql(skip)]
    committee_id: CommitteeId,
}

#[ComplexObject]
impl CommitteeThreshold {
    async fn committee(&self, ctx: &Context<'_>) -> async_graphql::Result<Committee> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let committee = app
            .governance()
            .find_committee_by_id(sub, self.committee_id)
            .await?
            .expect("committee not found");
        Ok(Committee::from(committee))
    }
}

// ── Committee ──────────────────────────────────────────────────────────

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Committee {
    id: ID,
    committee_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub entity: Arc<DomainCommittee>,
}

impl From<DomainCommittee> for Committee {
    fn from(committee: DomainCommittee) -> Self {
        Self {
            id: committee.id.to_global_id(),
            committee_id: committee.id.into(),
            created_at: committee.created_at().into(),
            entity: Arc::new(committee),
        }
    }
}

#[ComplexObject]
impl Committee {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn current_members(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let member_ids: Vec<UserId> = self
            .entity
            .members()
            .into_iter()
            .map(UserId::from)
            .collect();
        let users: std::collections::HashMap<UserId, User> =
            app.access().users().find_all(&member_ids).await?;
        Ok(users.into_values().collect())
    }
}

// ── Policy ─────────────────────────────────────────────────────────────

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Policy {
    id: ID,
    policy_id: UUID,
    approval_process_type: ApprovalProcessType,

    #[graphql(skip)]
    pub entity: Arc<DomainPolicy>,
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
