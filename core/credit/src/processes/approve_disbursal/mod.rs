mod job;

use tracing::instrument;

use ::job::Jobs;
use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{
    ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType, Governance, GovernanceAction,
    GovernanceEvent, GovernanceObject,
};

use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo, ledger::CreditLedger, obligation::Obligations,
    primitives::DisbursalId, CoreCreditAction, CoreCreditError, CoreCreditEvent, CoreCreditObject,
    Disbursal, Disbursals, LedgerTxId,
};

pub use job::*;
pub const APPROVE_DISBURSAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursal");

pub struct ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    disbursals: Disbursals<Perms, E>,
    obligations: Obligations<Perms, E>,
    credit_facility_repo: CreditFacilityRepo<E>,
    jobs: Jobs,
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
            disbursals: self.disbursals.clone(),
            obligations: self.obligations.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            jobs: self.jobs.clone(),
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
        disbursals: &Disbursals<Perms, E>,
        obligations: &Obligations<Perms, E>,
        credit_facility_repo: &CreditFacilityRepo<E>,
        jobs: &Jobs,
        governance: &Governance<Perms, E>,
        ledger: &CreditLedger,
    ) -> Self {
        Self {
            disbursals: disbursals.clone(),
            obligations: obligations.clone(),
            credit_facility_repo: credit_facility_repo.clone(),
            jobs: jobs.clone(),
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
        let mut db = self.disbursals.begin_op().await?;

        let tx_id = LedgerTxId::new();
        let (disbursal, data) = self
            .disbursals
            .conclude_approval_process_in_op(&mut db, id.into(), approved, tx_id)
            .await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(disbursal.facility_id)
            .await?;

        let span = tracing::Span::current();
        let new_obligation = if let Some(new_obligation) = data {
            new_obligation
        } else {
            span.record("already_applied", true);
            return Ok(disbursal);
        };
        span.record("already_applied", false);

        let obligation = if let Some(new_obligation) = new_obligation {
            let obligation = self
                .obligations
                .create_with_jobs_in_op(&mut db, new_obligation)
                .await?;
            Some(obligation)
        } else {
            None
        };
        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        if let Some(obligation) = obligation {
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
