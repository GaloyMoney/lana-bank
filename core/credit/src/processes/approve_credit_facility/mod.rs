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
    error::CoreCreditError, CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilities,
    CreditFacility, CreditFacilityId,
};

pub use job::*;
pub const APPROVE_CREDIT_FACILITY_PROCESS: ApprovalProcessType =
    ApprovalProcessType::new("credit-facility");

pub struct ApproveCreditFacility<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    credit_facilities: CreditFacilities<Perms, E>,
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
            credit_facilities: self.credit_facilities.clone(),
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
        repo: &CreditFacilities<Perms, E>,
        audit: &Perms::Audit,
        governance: &Governance<Perms, E>,
    ) -> Self {
        Self {
            credit_facilities: repo.clone(),
            audit: audit.clone(),
            governance: governance.clone(),
        }
    }

    pub async fn execute_from_svc(
        &self,
        credit_facility: &CreditFacility,
    ) -> Result<Option<CreditFacility>, CoreCreditError> {
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
    ) -> Result<CreditFacility, CoreCreditError> {
        let credit_facility = self.credit_facilities.approve(id.into(), approved).await?;
        Ok(credit_facility)
    }
}
