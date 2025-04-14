mod job;

use tracing::instrument;

use ::job::{JobId, Jobs};
use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::Idempotent;
use governance::{
    ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType, Governance, GovernanceAction,
    GovernanceEvent, GovernanceObject,
};

use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo, jobs::obligation_due, ledger::CreditLedger,
    obligation::Obligations, primitives::DisbursalId, CoreCreditAction, CoreCreditError,
    CoreCreditEvent, CoreCreditObject, Disbursal, DisbursalRepo, LedgerTxId,
};

pub use job::*;
pub const APPROVE_DISBURSAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursal");

pub struct ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    disbursal_repo: DisbursalRepo,
    obligations: Obligations<Perms, E>,
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
            obligations: self.obligations.clone(),
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
        obligations: &Obligations<Perms, E>,
        credit_facility_repo: &CreditFacilityRepo<E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
        governance: &Governance<Perms, E>,
        ledger: &CreditLedger,
    ) -> Self {
        Self {
            disbursal_repo: disbursal_repo.clone(),
            obligations: obligations.clone(),
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
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::disbursal(disbursal.id),
                CoreCreditAction::DISBURSAL_SETTLE,
            )
            .await?;

        let span = tracing::Span::current();
        let tx_id = LedgerTxId::new();
        let new_obligation = if let Idempotent::Executed(new_obligation) =
            disbursal.approval_process_concluded(tx_id, approved, audit_info.clone())
        {
            new_obligation
        } else {
            span.record("already_applied", true);
            return Ok(disbursal);
        };
        span.record("already_applied", false);

        let obligation = if let Some(new_obligation) = new_obligation {
            Some(
                self.obligations
                    .create_in_op(&mut db, new_obligation)
                    .await?,
            )
        } else {
            None
        };
        self.disbursal_repo
            .update_in_op(&mut db, &mut disbursal)
            .await?;
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        if let Some(obligation) = obligation {
            self.jobs
                .create_and_spawn_at_in_op(
                    &mut db,
                    JobId::new(),
                    obligation_due::CreditFacilityJobConfig::<Perms, E> {
                        obligation_id: obligation.id,
                        _phantom: std::marker::PhantomData,
                    },
                    obligation.due_at(),
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
