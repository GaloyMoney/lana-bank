mod job;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{
    ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType, Governance, GovernanceAction,
    GovernanceEvent, GovernanceObject,
};
use outbox::OutboxEventMarker;

use crate::{
    error::CreditFacilityError, CoreCreditAction, CoreCreditEvent, CoreCreditObject,
    CreditFacility, CreditFacilityId, CreditFacilityRepo,
};

pub use job::*;
pub const APPROVE_CREDIT_FACILITY_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility");

pub struct ApproveCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    repo: CreditFacilityRepo<E>,
    audit: Perms::Audit,
    governance: Governance<Perms, E>,
}

impl<Perms, E> Clone for ApproveCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            audit: self.audit.clone(),
            governance: self.governance.clone(),
        }
    }
}

impl<Perms, E> ApproveCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        repo: &CreditFacilityRepo<E>,
        audit: &Perms::Audit,
        governance: &Governance<Perms, E>,
    ) -> Self {
        Self {
            repo: repo.clone(),
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
                CoreCreditObject::credit_facility(credit_facility.id),
                CoreCreditAction::CREDIT_FACILITY_CONCLUDE_APPROVAL_PROCESS,
            )
            .await?;
        if credit_facility
            .approval_process_concluded(approved, audit_info)
            .was_already_applied()
        {
            return Ok(credit_facility);
        }

        self.repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;

        db.commit().await?;

        Ok(credit_facility)
    }
}
