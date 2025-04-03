mod job;

use tracing::instrument;

use ::job::Jobs;
use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::Idempotent;
use governance::{
    ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType, Governance, GovernanceAction,
    GovernanceEvent, GovernanceObject,
};

use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo, ledger::CreditLedger, obligation_overdue,
    primitives::DisbursalId, CoreCreditAction, CoreCreditError, CoreCreditEvent, CoreCreditObject,
    Disbursal, DisbursalRepo, LedgerTxId, ObligationRepo,
};

pub use job::*;
pub const APPROVE_DISBURSAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursal");

pub struct ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    disbursal_repo: DisbursalRepo,
    obligation_repo: ObligationRepo,
    credit_facility_repo: CreditFacilityRepo<E>,
    jobs: Jobs,
    audit: Perms::Audit,
    governance: Governance<Perms, E>,
    ledger: CreditLedger,
}

impl<Perms, E> Clone for ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            disbursal_repo: self.disbursal_repo.clone(),
            obligation_repo: self.obligation_repo.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
            governance: self.governance.clone(),
            ledger: self.ledger.clone(),
        }
    }
}

impl<Perms, E> ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        disbursal_repo: &DisbursalRepo,
        obligation_repo: &ObligationRepo,
        credit_facility_repo: &CreditFacilityRepo<E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
        governance: &Governance<Perms, E>,
        ledger: &CreditLedger,
    ) -> Self {
        Self {
            disbursal_repo: disbursal_repo.clone(),
            obligation_repo: obligation_repo.clone(),
            credit_facility_repo: credit_facility_repo.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
            governance: governance.clone(),
            ledger: ledger.clone(),
        }
    }

    pub async fn execute_from_svc(
        &self,
        disbursal: &Disbursal,
    ) -> Result<Option<Disbursal>, CoreCreditError> {
        if disbursal.is_approval_process_concluded() {
            return Ok(None);
        }

        let process: ApprovalProcess = self
            .governance
            .find_all_approval_processes(&[disbursal.approval_process_id])
            .await?
            .remove(&disbursal.approval_process_id)
            .expect("approval process not found");

        let res = match process.status() {
            ApprovalProcessStatus::Approved => Some(self.execute(disbursal.id, true).await?),
            ApprovalProcessStatus::Denied => Some(self.execute(disbursal.id, false).await?),
            _ => None,
        };
        Ok(res)
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    #[instrument(
        name = "credit_facility.approve_disbursal",
        skip(self),
        fields(already_applied, disbursal_executed)
    )]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<DisbursalId>,
        approved: bool,
    ) -> Result<Disbursal, CoreCreditError> {
        let mut disbursal = self.disbursal_repo.find_by_id(id.into()).await?;
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(disbursal.facility_id)
            .await?;

        let mut db = self.disbursal_repo.begin_op().await?;
        let executed_at = db.now();
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::disbursal(disbursal.id),
                CoreCreditAction::DISBURSAL_CONCLUDE_APPROVAL_PROCESS,
            )
            .await?;
        let disbursal_audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                // NOTE: change to DisbursalObject
                CoreCreditObject::disbursal(disbursal.id),
                CoreCreditAction::DISBURSAL_SETTLE,
            )
            .await?;

        let span = tracing::Span::current();
        let tx_id = LedgerTxId::new();
        let new_obligation = if let Idempotent::Executed(new_obligation) =
            disbursal.approval_process_concluded(tx_id, approved, audit_info.clone())
        {
            let obligation_id = new_obligation.as_ref().map(|n| n.id());
            if credit_facility
                .disbursal_concluded(
                    &disbursal,
                    tx_id,
                    obligation_id,
                    executed_at,
                    disbursal_audit_info,
                )
                .was_ignored()
            {
                return Ok(disbursal);
            }

            new_obligation
        } else {
            span.record("already_applied", true);
            return Ok(disbursal);
        };
        span.record("already_applied", false);

        self.disbursal_repo
            .update_in_op(&mut db, &mut disbursal)
            .await?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        if let Some(new_obligation) = new_obligation {
            let obligation = self
                .obligation_repo
                .create_in_op(&mut db, new_obligation)
                .await?;

            self.jobs
                .create_and_spawn_at_in_op(
                    &mut db,
                    obligation.id,
                    obligation_overdue::CreditFacilityJobConfig::<Perms> {
                        obligation_id: obligation.id,
                        _phantom: std::marker::PhantomData,
                    },
                    obligation.overdue_at(),
                )
                .await?;

            self.ledger
                .settle_disbursal(
                    db,
                    obligation,
                    credit_facility.account_ids.facility_account_id,
                )
                .await?;
        } else {
            self.ledger
                .cancel_disbursal(
                    db,
                    tx_id,
                    disbursal.amount,
                    credit_facility.account_ids.facility_account_id,
                )
                .await?;
        }

        Ok(disbursal)
    }
}
