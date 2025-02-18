mod job;

use tracing::instrument;

use es_entity::Idempotent;
use governance::{ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType};

use crate::{
    audit::{Audit, AuditSvc},
    credit_facility::{
        error::CreditFacilityError, ledger::CreditLedger, repo::CreditFacilityRepo, Disbursal,
        DisbursalRepo,
    },
    governance::Governance,
    primitives::DisbursalId,
};
use rbac_types::{AppObject, CreditFacilityAction};

pub use job::*;

pub const APPROVE_DISBURSAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursal");

#[derive(Clone)]
pub struct ApproveDisbursal {
    disbursal_repo: DisbursalRepo,
    credit_facility_repo: CreditFacilityRepo,
    audit: Audit,
    governance: Governance,
    ledger: CreditLedger,
}

impl ApproveDisbursal {
    pub(in crate::credit_facility) fn new(
        disbursal_repo: &DisbursalRepo,
        credit_facility_repo: &CreditFacilityRepo,
        audit: &Audit,
        governance: &Governance,
        ledger: &CreditLedger,
    ) -> Self {
        Self {
            disbursal_repo: disbursal_repo.clone(),
            credit_facility_repo: credit_facility_repo.clone(),
            audit: audit.clone(),
            governance: governance.clone(),
            ledger: ledger.clone(),
        }
    }

    pub async fn execute_from_svc(
        &self,
        disbursal: &Disbursal,
    ) -> Result<Option<Disbursal>, CreditFacilityError> {
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
    ) -> Result<Disbursal, CreditFacilityError> {
        let mut disbursal = self.disbursal_repo.find_by_id(id.into()).await?;
        let mut db = self.disbursal_repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                AppObject::CreditFacility,
                CreditFacilityAction::ConcludeDisbursalApprovalProcess,
            )
            .await?;
        let span = tracing::Span::current();
        let Idempotent::Executed(disbursal_data) =
            disbursal.approval_process_concluded(approved, audit_info.clone())
        else {
            span.record("already_applied", true);
            return Ok(disbursal);
        };
        span.record("already_applied", false);

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(disbursal.facility_id)
            .await?;

        let executed_at = db.now();
        let disbursal_audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                AppObject::CreditFacility,
                CreditFacilityAction::SettleDisbursal,
            )
            .await?;

        let (now, mut tx) = (db.now(), db.into_tx());
        let sub_op = {
            use sqlx::Acquire;
            es_entity::DbOp::new(tx.begin().await?, now)
        };

        if let Idempotent::Executed(_) = credit_facility.disbursal_concluded(
            &disbursal,
            Some(disbursal_data.tx_id),
            executed_at,
            disbursal_audit_info,
        ) {
            self.ledger
                .conclude_disbursal(sub_op, disbursal_data)
                .await?;

            let mut db = es_entity::DbOp::new(tx, now);
            self.disbursal_repo
                .update_in_op(&mut db, &mut disbursal)
                .await?;
            self.credit_facility_repo
                .update_in_op(&mut db, &mut credit_facility)
                .await?;
            db.commit().await?;
        }
        Ok(disbursal)
    }
}
