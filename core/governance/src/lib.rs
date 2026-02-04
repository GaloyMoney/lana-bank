#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod approval_process;
mod committee;
pub mod error;
mod policy;
mod primitives;
pub mod public;

use es_entity::clock::ClockHandle;
use tracing::instrument;
use tracing_macros::record_error_severity;

use std::collections::{HashMap, HashSet};

use audit::AuditSvc;
use authz::PermissionCheck;
use obix::out::{Outbox, OutboxEventMarker};

pub use approval_process::{error as approval_process_error, *};
pub use committee::{error as committee_error, *};
use error::*;
use policy::error::PolicyError;
pub use policy::{error as policy_error, *};
pub use primitives::*;
pub use public::*;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::approval_process::ApprovalProcessEvent;
    pub use crate::committee::CommitteeEvent;
    pub use crate::policy::PolicyEvent;
}

pub struct Governance<Perms, E>
where
    Perms: PermissionCheck,
    E: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static + Unpin,
{
    committee_repo: CommitteeRepo,
    policy_repo: PolicyRepo,
    process_repo: ApprovalProcessRepo,
    authz: Perms,
    outbox: Outbox<E>,
}

impl<Perms, E> Clone for Governance<Perms, E>
where
    Perms: PermissionCheck,
    E: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static + Unpin,
{
    fn clone(&self) -> Self {
        Self {
            committee_repo: self.committee_repo.clone(),
            policy_repo: self.policy_repo.clone(),
            process_repo: self.process_repo.clone(),
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
        }
    }
}

impl<Perms, E> Governance<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, outbox: &Outbox<E>, clock: ClockHandle) -> Self {
        let committee_repo = CommitteeRepo::new(pool, clock.clone());
        let policy_repo = PolicyRepo::new(pool, clock.clone());
        let process_repo = ApprovalProcessRepo::new(pool, clock);

        Self {
            committee_repo,
            policy_repo,
            process_repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "governance.init_policy", skip(self), fields(process_type = ?process_type, policy_id = tracing::field::Empty))]
    pub async fn init_policy(
        &self,
        process_type: ApprovalProcessType,
    ) -> Result<Policy, GovernanceError> {
        let mut db = self.policy_repo.begin_op().await?;
        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut db,
                GovernanceObject::all_policies(),
                GovernanceAction::POLICY_CREATE,
            )
            .await?;

        let new_policy = NewPolicy::builder()
            .id(PolicyId::new())
            .process_type(process_type.clone())
            .rules(ApprovalRules::SystemAutoApprove)
            .build()
            .expect("Could not build new policy");

        match self.policy_repo.create_in_op(&mut db, new_policy).await {
            Ok(policy) => {
                db.commit().await?;
                Ok(policy)
            }
            Err(PolicyError::DuplicateApprovalProcessType) => {
                let policy = self.policy_repo.find_by_process_type(process_type).await?;
                Ok(policy)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(name = "governance.find_policy", skip(self))]
    pub async fn find_policy(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<PolicyId> + std::fmt::Debug,
    ) -> Result<Option<Policy>, GovernanceError> {
        let policy_id = id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::policy(policy_id),
                GovernanceAction::POLICY_READ,
            )
            .await?;

        self.policy_repo
            .maybe_find_by_id(policy_id)
            .await
            .map_err(GovernanceError::PolicyError)
    }

    #[record_error_severity]
    #[instrument(name = "governance.list_policies", skip(self))]
    pub async fn list_policies_by_created_at(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<policy_cursor::PoliciesByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Policy, policy_cursor::PoliciesByCreatedAtCursor>,
        GovernanceError,
    > {
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::all_policies(),
                GovernanceAction::POLICY_LIST,
            )
            .await?;
        let policies = self
            .policy_repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?;

        Ok(policies)
    }

    #[record_error_severity]
    #[instrument(name = "governance.assign_committee_to_policy", skip(self))]
    pub async fn assign_committee_to_policy(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        policy_id: impl Into<PolicyId> + std::fmt::Debug,
        committee_id: impl Into<CommitteeId> + std::fmt::Debug,
        threshold: usize,
    ) -> Result<Policy, GovernanceError> {
        let policy_id = policy_id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::policy(policy_id),
                GovernanceAction::POLICY_UPDATE_RULES,
            )
            .await?;

        let committee_id = committee_id.into();
        let committee = self.committee_repo.find_by_id(committee_id).await?;

        let mut policy = self.policy_repo.find_by_id(policy_id).await?;
        if policy
            .assign_committee(committee.id, committee.n_members(), threshold)?
            .did_execute()
        {
            let mut db_tx = self.policy_repo.begin_op().await?;
            self.policy_repo
                .update_in_op(&mut db_tx, &mut policy)
                .await?;
            db_tx.commit().await?;
        }

        Ok(policy)
    }

    #[record_error_severity]
    #[instrument(name = "governance.find_all_policies", skip(self))]
    pub async fn find_all_policies<T: From<Policy>>(
        &self,
        ids: &[PolicyId],
    ) -> Result<HashMap<PolicyId, T>, GovernanceError> {
        Ok(self.policy_repo.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "governance.start_process_in_op", skip(self, db))]
    pub async fn start_process_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: impl Into<ApprovalProcessId> + std::fmt::Debug,
        target_ref: String,
        process_type: ApprovalProcessType,
    ) -> Result<ApprovalProcess, GovernanceError> {
        let policy = self.policy_repo.find_by_process_type(process_type).await?;
        self.authz
            .audit()
            .record_system_entry(
                GovernanceObject::all_approval_processes(),
                GovernanceAction::APPROVAL_PROCESS_CREATE,
            )
            .await?;
        let new_process = policy.spawn_process(id.into(), target_ref);
        let mut process = self.process_repo.create_in_op(db, new_process).await?;
        let eligible = self.eligible_voters_for_process(&process).await?;
        if self
            .maybe_fire_concluded_event_in_op(db, eligible, &mut process)
            .await?
        {
            self.process_repo.update_in_op(db, &mut process).await?;
        }
        Ok(process)
    }

    #[record_error_severity]
    #[instrument(name = "governance.approve_process", skip(self))]
    pub async fn approve_process(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process_id: impl Into<ApprovalProcessId> + std::fmt::Debug,
    ) -> Result<ApprovalProcess, GovernanceError>
    where
        CommitteeMemberId:
            for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let process_id = process_id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::all_approval_processes(),
                GovernanceAction::APPROVAL_PROCESS_APPROVE,
            )
            .await?;
        let member_id = CommitteeMemberId::try_from(sub)
            .map_err(|_| GovernanceError::SubjectIsNotCommitteeMember)?;
        let mut process = self.process_repo.find_by_id(process_id).await?;
        let eligible = self.eligible_voters_for_process(&process).await?;

        if process.approve(&eligible, member_id).did_execute() {
            let mut db = self.policy_repo.begin_op().await?;
            self.maybe_fire_concluded_event_in_op(&mut db, eligible, &mut process)
                .await?;
            self.process_repo
                .update_in_op(&mut db, &mut process)
                .await?;
            db.commit().await?;
        }

        Ok(process)
    }

    #[record_error_severity]
    #[instrument(name = "governance.deny_process", skip(self))]
    pub async fn deny_process(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process_id: impl Into<ApprovalProcessId> + std::fmt::Debug,
        reason: String,
    ) -> Result<ApprovalProcess, GovernanceError>
    where
        CommitteeMemberId:
            for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let process_id = process_id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::approval_process(process_id),
                GovernanceAction::APPROVAL_PROCESS_DENY,
            )
            .await?;
        let member_id = CommitteeMemberId::try_from(sub)
            .map_err(|_| GovernanceError::SubjectIsNotCommitteeMember)?;
        let mut process = self.process_repo.find_by_id(process_id).await?;
        let eligible = if let Some(committee_id) = process.committee_id() {
            self.committee_repo
                .find_by_id(committee_id)
                .await?
                .members()
        } else {
            HashSet::new()
        };
        if process.deny(&eligible, member_id, reason).did_execute() {
            let mut db = self.policy_repo.begin_op().await?;
            self.maybe_fire_concluded_event_in_op(&mut db, eligible, &mut process)
                .await?;
            self.process_repo
                .update_in_op(&mut db, &mut process)
                .await?;
            db.commit().await?;
        }

        Ok(process)
    }

    #[record_error_severity]
    #[instrument(name = "governance.create_committee", skip(self, name), fields(committee_name = %name))]
    pub async fn create_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
    ) -> Result<Committee, GovernanceError> {
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::all_committees(),
                GovernanceAction::COMMITTEE_CREATE,
            )
            .await?;

        let new_committee = NewCommittee::builder()
            .id(CommitteeId::new())
            .name(name)
            .build()
            .expect("Could not build new committee");

        let mut db = self.committee_repo.begin_op().await?;
        let committee = self
            .committee_repo
            .create_in_op(&mut db, new_committee)
            .await?;
        db.commit().await?;
        Ok(committee)
    }

    async fn maybe_fire_concluded_event_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        eligible: HashSet<CommitteeMemberId>,
        process: &mut ApprovalProcess,
    ) -> Result<bool, GovernanceError> {
        self.authz
            .audit()
            .record_system_entry_in_op(
                op,
                GovernanceObject::approval_process(process.id),
                GovernanceAction::APPROVAL_PROCESS_CONCLUDE,
            )
            .await?;

        if let es_entity::Idempotent::Executed(_) = process.check_concluded(eligible) {
            let entity = PublicApprovalProcess::from(&*process);
            self.outbox
                .publish_all_persisted(op, [GovernanceEvent::ApprovalProcessConcluded { entity }])
                .await?;

            return Ok(true);
        }

        Ok(false)
    }

    #[record_error_severity]
    #[instrument(name = "governance.add_member_to_committee", skip(self))]
    pub async fn add_member_to_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        committee_id: impl Into<CommitteeId> + std::fmt::Debug,
        member_id: impl Into<CommitteeMemberId> + std::fmt::Debug,
    ) -> Result<Committee, GovernanceError> {
        let committee_id = committee_id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::committee(committee_id),
                GovernanceAction::COMMITTEE_ADD_MEMBER,
            )
            .await?;

        let mut committee = self.committee_repo.find_by_id(committee_id).await?;
        if committee.add_member(member_id.into()).did_execute() {
            self.committee_repo.update(&mut committee).await?;
        }

        Ok(committee)
    }

    #[record_error_severity]
    #[instrument(name = "governance.remove_member_from_committee", skip(self))]
    pub async fn remove_member_from_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        committee_id: impl Into<CommitteeId> + std::fmt::Debug,
        member_id: impl Into<CommitteeMemberId> + std::fmt::Debug,
    ) -> Result<Committee, GovernanceError> {
        let committee_id = committee_id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::committee(committee_id),
                GovernanceAction::COMMITTEE_REMOVE_MEMBER,
            )
            .await?;

        let mut committee = self.committee_repo.find_by_id(committee_id).await?;
        if committee.remove_member(member_id.into()).did_execute() {
            self.committee_repo.update(&mut committee).await?;
        }

        Ok(committee)
    }

    #[record_error_severity]
    #[instrument(name = "governance.find_committee_by_id", skip(self))]
    pub async fn find_committee_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        committee_id: impl Into<CommitteeId> + std::fmt::Debug,
    ) -> Result<Option<Committee>, GovernanceError> {
        let committee_id = committee_id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::committee(committee_id),
                GovernanceAction::COMMITTEE_READ,
            )
            .await?;

        self.committee_repo
            .maybe_find_by_id(committee_id)
            .await
            .map_err(GovernanceError::CommitteeError)
    }

    #[record_error_severity]
    #[instrument(name = "governance.list_committees", skip(self))]
    pub async fn list_committees(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<
            committee::committee_cursor::CommitteesByCreatedAtCursor,
        >,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            Committee,
            committee::committee_cursor::CommitteesByCreatedAtCursor,
        >,
        GovernanceError,
    > {
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::all_committees(),
                GovernanceAction::COMMITTEE_LIST,
            )
            .await?;

        let committees = self
            .committee_repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?;
        Ok(committees)
    }

    #[record_error_severity]
    #[instrument(name = "governance.find_all_committees", skip(self))]
    pub async fn find_all_committees<T: From<Committee>>(
        &self,
        ids: &[CommitteeId],
    ) -> Result<HashMap<CommitteeId, T>, GovernanceError> {
        Ok(self.committee_repo.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "governance.find_approval_process_by_id", skip(self))]
    pub async fn find_approval_process_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process_id: impl Into<ApprovalProcessId> + std::fmt::Debug,
    ) -> Result<Option<ApprovalProcess>, GovernanceError> {
        let process_id = process_id.into();
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::approval_process(process_id),
                GovernanceAction::APPROVAL_PROCESS_READ,
            )
            .await?;

        self.process_repo
            .maybe_find_by_id(process_id)
            .await
            .map_err(GovernanceError::ApprovalProcessError)
    }

    #[record_error_severity]
    #[instrument(name = "governance.list_approval_processes", skip(self))]
    pub async fn list_approval_processes(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<
            approval_process_cursor::ApprovalProcessesByCreatedAtCursor,
        >,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            ApprovalProcess,
            approval_process_cursor::ApprovalProcessesByCreatedAtCursor,
        >,
        GovernanceError,
    > {
        self.authz
            .enforce_permission(
                sub,
                GovernanceObject::all_approval_processes(),
                GovernanceAction::APPROVAL_PROCESS_LIST,
            )
            .await?;

        let approval_processes = self
            .process_repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?;
        Ok(approval_processes)
    }

    #[record_error_severity]
    #[instrument(name = "governance.find_all_approval_processes", skip(self))]
    pub async fn find_all_approval_processes<T: From<ApprovalProcess>>(
        &self,
        ids: &[ApprovalProcessId],
    ) -> Result<HashMap<ApprovalProcessId, T>, GovernanceError> {
        Ok(self.process_repo.find_all(ids).await?)
    }

    pub async fn subject_can_submit_decision(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process: &ApprovalProcess,
        committee: Option<&Committee>,
    ) -> Result<bool, GovernanceError>
    where
        CommitteeMemberId:
            for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        if let Some(committee) = committee {
            let member_id = CommitteeMemberId::try_from(sub)
                .map_err(|_| GovernanceError::SubjectIsNotCommitteeMember)?;
            Ok(process.can_member_vote(member_id, committee.members()))
        } else {
            Ok(false)
        }
    }

    async fn eligible_voters_for_process(
        &self,
        process: &ApprovalProcess,
    ) -> Result<HashSet<CommitteeMemberId>, GovernanceError> {
        let res = if let Some(committee_id) = process.committee_id() {
            self.committee_repo
                .find_by_id(committee_id)
                .await?
                .members()
        } else {
            HashSet::new()
        };
        Ok(res)
    }
}
