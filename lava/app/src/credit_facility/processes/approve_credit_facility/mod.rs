mod job;

use tracing::instrument;

use governance::{ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType};

use crate::{
    audit::{Audit, AuditSvc},
    credit_facility::{activate, error::CreditFacilityError, CreditFacility, CreditFacilityRepo},
    governance::Governance,
    job::Jobs,
    ledger::Ledger,
    price::Price,
    primitives::CreditFacilityId,
};
use rbac_types::{AppObject, CreditFacilityAction};

pub use job::*;

pub const APPROVE_CREDIT_FACILITY_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility");

#[derive(Clone)]
pub struct ApproveCreditFacility {
    repo: CreditFacilityRepo,
    ledger: Ledger,
    jobs: Jobs,
    governance: Governance,
    audit: Audit,
    price: Price,
}

impl ApproveCreditFacility {
    pub(in crate::credit_facility) fn new(
        repo: &CreditFacilityRepo,
        ledger: &Ledger,
        price: &Price,
        jobs: &Jobs,
        audit: &Audit,
        governance: &Governance,
    ) -> Self {
        Self {
            repo: repo.clone(),
            ledger: ledger.clone(),
            price: price.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
            governance: governance.clone(),
        }
    }

    pub async fn execute_from_svc(
        &self,
        credit_facility: &CreditFacility,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        if credit_facility.is_approval_process_concluded() {
            return Ok(None);
        }

        let process: ApprovalProcess = self
            .governance
            .find_all_approval_processes(&[credit_facility.approval_process_id])
            .await?
            .remove(&credit_facility.approval_process_id)
            .expect("approval process not found");

        let res = match process.status() {
            ApprovalProcessStatus::Approved => Some(self.execute(credit_facility.id, true).await?),
            ApprovalProcessStatus::Denied => Some(self.execute(credit_facility.id, false).await?),
            _ => None,
        };
        Ok(res)
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    #[instrument(name = "credit_facility.approval.execute", skip(self))]
    pub async fn execute(
        &self,
        id: impl es_entity::RetryableInto<CreditFacilityId>,
        approved: bool,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let mut credit_facility = self.repo.find_by_id(id.into()).await?;
        if credit_facility.is_approval_process_concluded() {
            return Ok(credit_facility);
        }
        let mut db = self.repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                AppObject::CreditFacility,
                CreditFacilityAction::ConcludeApprovalProcess,
            )
            .await?;
        if credit_facility
            .approval_process_concluded(approved, audit_info)
            .was_already_applied()
        {
            return Ok(credit_facility);
        }

        let price = self.price.usd_cents_per_btc().await?;
        activate::execute(
            &mut credit_facility,
            &mut db,
            &self.ledger,
            &self.audit,
            &self.jobs,
            price,
        )
        .await?;
        if self
            .repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?
        {
            db.commit().await?;
        }
        Ok(credit_facility)
    }
}
