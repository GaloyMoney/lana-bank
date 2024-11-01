#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod approval_process;
mod committee;
pub mod error;
mod event;
mod policy;
mod primitives;

use sqlx::Acquire;
use tracing::instrument;

use std::collections::{HashMap, HashSet};

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

pub use approval_process::{error as approval_process_error, *};
pub use committee::{error as committee_error, *};
use error::*;
pub use event::*;
pub use policy::{error as policy_error, *};
pub use primitives::*;

pub struct Governance<Perms, E>
where
    Perms: PermissionCheck,
    E: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static + Unpin,
{
    pool: sqlx::PgPool,
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
            pool: self.pool.clone(),
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
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, outbox: &Outbox<E>) -> Self {
        let committee_repo = CommitteeRepo::new(pool);
        let policy_repo = PolicyRepo::new(pool);
        let process_repo = ApprovalProcessRepo::new(pool);

        Self {
            pool: pool.clone(),
            committee_repo,
            policy_repo,
            process_repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
        }
    }

    pub async fn init_policy(
        &self,
        process_type: ApprovalProcessType,
    ) -> Result<Policy, GovernanceError> {
        let mut db = self.pool.begin().await?;
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                &mut db,
                GovernanceObject::all_policies(),
                GovernanceAction::POLICY_CREATE,
            )
            .await?;

        let new_policy = NewPolicy::builder()
            .id(PolicyId::new())
            .process_type(process_type)
            .rules(ApprovalRules::SystemAutoApprove)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new policy");

        let policy = self.policy_repo.create_in_tx(&mut db, new_policy).await?;
        db.commit().await?;
        Ok(policy)
    }

    #[instrument(name = "governance.find_policy", skip(self), err)]
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

        match self.policy_repo.find_by_id(policy_id).await {
            Ok(policy) => Ok(Some(policy)),
            Err(PolicyError::NotFound) => Ok(None),
            Err(e) => Err(GovernanceError::PolicyError(e)),
        }
    }

    #[instrument(name = "governance.list_policies", skip(self), err)]
    pub async fn list_policies_by_created_at(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<policy_cursor::PolicyByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Policy, policy_cursor::PolicyByCreatedAtCursor>,
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

    #[instrument(name = "governance.assign_committee_to_policy", skip(self), err)]
    pub async fn assign_committee_to_policy(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        policy_id: impl Into<PolicyId> + std::fmt::Debug,
        committee_id: impl Into<CommitteeId> + std::fmt::Debug,
        threshold: usize,
    ) -> Result<Policy, GovernanceError> {
        let policy_id = policy_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                GovernanceObject::policy(policy_id),
                GovernanceAction::POLICY_UPDATE_RULES,
            )
            .await?;

        let committee_id = committee_id.into();
        let committee = self.committee_repo.find_by_id(committee_id).await?;
        let mut policy = self.policy_repo.find_by_id(policy_id).await?;
        policy.assign_committee(committee.id, threshold, audit_info);

        let mut db_tx = self.pool.begin().await?;
        self.policy_repo
            .update_in_tx(&mut db_tx, &mut policy)
            .await?;
        db_tx.commit().await?;

        Ok(policy)
    }

    #[instrument(name = "governance.find_all_policies", skip(self), err)]
    pub async fn find_all_policies<T: From<Policy>>(
        &self,
        ids: &[PolicyId],
    ) -> Result<HashMap<PolicyId, T>, PolicyError> {
        self.policy_repo.find_all(ids).await
    }

    #[instrument(name = "governance.start_process", skip(self), err)]
    pub async fn start_process(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: impl Into<ApprovalProcessId> + std::fmt::Debug,
        target_ref: String,
        process_type: ApprovalProcessType,
    ) -> Result<ApprovalProcess, GovernanceError> {
        let policy = self.policy_repo.find_by_process_type(process_type).await?;
        let audit_info = self
            .authz
            .audit()
            .record_system_entry(
                GovernanceObject::all_approval_processes(),
                GovernanceAction::APPROVAL_PROCESS_CREATE,
            )
            .await?;
        let process = policy.spawn_process(id.into(), target_ref, audit_info);
        let mut process = self.process_repo.create_in_tx(db, process).await?;
        let eligible = self.eligible_voters_for_process(&process).await?;
        if self
            .maybe_fire_concluded_event(db.begin().await?, eligible, &mut process)
            .await?
        {
            self.process_repo.update_in_tx(db, &mut process).await?;
        }
        Ok(process)
    }

    #[instrument(name = "governance.approve_process", skip(self), err)]
    pub async fn approve_process(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process_id: impl Into<ApprovalProcessId> + std::fmt::Debug,
    ) -> Result<ApprovalProcess, GovernanceError>
    where
        UserId: for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let process_id = process_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                GovernanceObject::all_approval_processes(),
                GovernanceAction::APPROVAL_PROCESS_APPROVE,
            )
            .await?;
        let user_id = UserId::try_from(sub).map_err(|_| GovernanceError::SubjectIsNotUser)?;
        let mut process = self.process_repo.find_by_id(process_id).await?;
        let eligible = self.eligible_voters_for_process(&process).await?;
        process.approve(&eligible, user_id, audit_info)?;
        let mut db = self.pool.begin().await?;
        self.maybe_fire_concluded_event(db.begin().await?, eligible, &mut process)
            .await?;
        self.process_repo
            .update_in_tx(&mut db, &mut process)
            .await?;
        db.commit().await?;

        Ok(process)
    }

    #[instrument(name = "governance.deny_process", skip(self), err)]
    pub async fn deny_process(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process_id: impl Into<ApprovalProcessId> + std::fmt::Debug,
    ) -> Result<ApprovalProcess, GovernanceError>
    where
        UserId: for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let process_id = process_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                GovernanceObject::approval_process(process_id),
                GovernanceAction::APPROVAL_PROCESS_DENY,
            )
            .await?;
        let user_id = UserId::try_from(sub).map_err(|_| GovernanceError::SubjectIsNotUser)?;
        let mut process = self.process_repo.find_by_id(process_id).await?;
        let eligible = if let Some(committee_id) = process.committee_id() {
            self.committee_repo
                .find_by_id(committee_id)
                .await?
                .members()
        } else {
            HashSet::new()
        };
        process.deny(&eligible, user_id, audit_info)?;
        let mut db = self.pool.begin().await?;
        self.maybe_fire_concluded_event(db.begin().await?, eligible, &mut process)
            .await?;
        self.process_repo
            .update_in_tx(&mut db, &mut process)
            .await?;
        db.commit().await?;

        Ok(process)
    }

    #[instrument(name = "governance.create_committee", skip(self), err)]
    pub async fn create_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
    ) -> Result<Committee, GovernanceError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                GovernanceObject::all_committees(),
                GovernanceAction::COMMITTEE_CREATE,
            )
            .await?;

        let new_committee = NewCommittee::builder()
            .id(CommitteeId::new())
            .name(name)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new committee");

        let mut db = self.pool.begin().await?;
        let committee = self
            .committee_repo
            .create_in_tx(&mut db, new_committee)
            .await?;
        db.commit().await?;
        Ok(committee)
    }

    async fn maybe_fire_concluded_event(
        &self,
        mut db: sqlx::Transaction<'_, sqlx::Postgres>,
        eligible: HashSet<UserId>,
        process: &mut ApprovalProcess,
    ) -> Result<bool, GovernanceError> {
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                &mut db,
                GovernanceObject::approval_process(process.id),
                GovernanceAction::APPROVAL_PROCESS_CONCLUDE,
            )
            .await?;
        let res = if let Some(approved) = process.check_concluded(eligible, audit_info) {
            self.outbox
                .persist(
                    &mut db,
                    GovernanceEvent::ApprovalProcessConcluded {
                        id: process.id,
                        approved,
                        process_type: process.process_type.clone(),
                        target_ref: process.target_ref().to_string(),
                    },
                )
                .await?;
            db.commit().await?;
            true
        } else {
            false
        };
        Ok(res)
    }

    #[instrument(name = "governance.add_user_to_committee", skip(self), err)]
    pub async fn add_user_to_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        committee_id: impl Into<CommitteeId> + std::fmt::Debug,
        user_id: impl Into<UserId> + std::fmt::Debug,
    ) -> Result<Committee, GovernanceError> {
        let committee_id = committee_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                GovernanceObject::committee(committee_id),
                GovernanceAction::COMMITTEE_ADD_USER,
            )
            .await?;

        let mut committee = self.committee_repo.find_by_id(committee_id).await?;
        committee.add_user(user_id.into(), audit_info)?;
        self.committee_repo.update(&mut committee).await?;

        Ok(committee)
    }

    #[instrument(name = "governance.remove_user_from_committee", skip(self), err)]
    pub async fn remove_user_from_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        committee_id: impl Into<CommitteeId> + std::fmt::Debug,
        user_id: impl Into<UserId> + std::fmt::Debug,
    ) -> Result<Committee, GovernanceError> {
        let committee_id = committee_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                GovernanceObject::committee(committee_id),
                GovernanceAction::COMMITTEE_REMOVE_USER,
            )
            .await?;

        let mut committee = self.committee_repo.find_by_id(committee_id).await?;
        committee.remove_user(user_id.into(), audit_info);
        self.committee_repo.update(&mut committee).await?;

        Ok(committee)
    }

    #[instrument(name = "governance.find_committee_by_id", skip(self), err)]
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

        match self.committee_repo.find_by_id(committee_id).await {
            Ok(committee) => Ok(Some(committee)),
            Err(CommitteeError::NotFound) => Ok(None),
            Err(e) => Err(GovernanceError::CommitteeError(e)),
        }
    }

    #[instrument(name = "governance.list_committees", skip(self), err)]
    pub async fn list_committees(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<
            committee::committee_cursor::CommitteeByCreatedAtCursor,
        >,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            Committee,
            committee::committee_cursor::CommitteeByCreatedAtCursor,
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

    #[instrument(name = "governance.find_all_committees", skip(self), err)]
    pub async fn find_all_committees<T: From<Committee>>(
        &self,
        ids: &[CommitteeId],
    ) -> Result<HashMap<CommitteeId, T>, CommitteeError> {
        self.committee_repo.find_all(ids).await
    }

    #[instrument(name = "governance.find_approval_process_by_id", skip(self), err)]
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

        match self.process_repo.find_by_id(process_id).await {
            Ok(process) => Ok(Some(process)),
            Err(ApprovalProcessError::NotFound) => Ok(None),
            Err(e) => Err(GovernanceError::ApprovalProcessError(e)),
        }
    }

    #[instrument(name = "governance.list_approval_processes", skip(self), err)]
    pub async fn list_approval_processes(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<
            approval_process_cursor::ApprovalProcessByCreatedAtCursor,
        >,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            ApprovalProcess,
            approval_process_cursor::ApprovalProcessByCreatedAtCursor,
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

    #[instrument(name = "governance.find_all_committees", skip(self), err)]
    pub async fn find_all_approval_processes<T: From<ApprovalProcess>>(
        &self,
        ids: &[ApprovalProcessId],
    ) -> Result<HashMap<ApprovalProcessId, T>, ApprovalProcessError> {
        self.process_repo.find_all(ids).await
    }

    pub async fn subject_can_submit_decision(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process: &ApprovalProcess,
        committee: Option<&Committee>,
    ) -> Result<bool, GovernanceError>
    where
        UserId: for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        if let Some(committee) = committee {
            let user_id = UserId::try_from(sub).map_err(|_| GovernanceError::SubjectIsNotUser)?;
            Ok(process.can_user_vote(user_id, committee.members()))
        } else {
            Ok(false)
        }
    }

    async fn eligible_voters_for_process(
        &self,
        process: &ApprovalProcess,
    ) -> Result<HashSet<UserId>, GovernanceError> {
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
