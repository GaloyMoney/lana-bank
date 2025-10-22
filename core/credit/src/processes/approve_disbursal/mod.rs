mod job;

use std::sync::Arc;

use ::job::Jobs;
use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{
    ApprovalProcessType, Governance, GovernanceAction, GovernanceEvent, GovernanceObject,
};
use tracing::instrument;

use outbox::OutboxEventMarker;

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{
    CoreCreditAction, CoreCreditError, CoreCreditEvent, CoreCreditObject, Disbursal, Disbursals,
    credit_facility::CreditFacilities, ledger::CreditLedger, primitives::DisbursalId,
};

pub use job::*;
pub const APPROVE_DISBURSAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursal");

pub struct ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    disbursals: Arc<Disbursals<Perms, E>>,
    credit_facilities: Arc<CreditFacilities<Perms, E>>,
    jobs: Arc<Jobs>,
    governance: Arc<Governance<Perms, E>>,
    ledger: Arc<CreditLedger>,
}

impl<Perms, E> Clone for ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            disbursals: self.disbursals.clone(),
            credit_facilities: self.credit_facilities.clone(),
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
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        disbursals: Arc<Disbursals<Perms, E>>,
        credit_facilities: Arc<CreditFacilities<Perms, E>>,
        jobs: Arc<Jobs>,
        governance: Arc<Governance<Perms, E>>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            disbursals,
            credit_facilities,
            jobs,
            governance,
            ledger,
        }
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
        let mut op = self.disbursals.begin_op().await?.with_db_time().await?;

        let disbursal = match self
            .disbursals
            .conclude_approval_process_in_op(&mut op, id.into(), approved)
            .await?
        {
            crate::ApprovalProcessOutcome::Ignored(disbursal) => {
                tracing::Span::current().record("already_applied", true);
                disbursal
            }
            crate::ApprovalProcessOutcome::Approved((disbursal, obligation)) => {
                tracing::Span::current().record("already_applied", false);

                let credit_facility = self
                    .credit_facilities
                    .find_by_id_without_audit(disbursal.facility_id) // changed for now
                    .await?;
                self.ledger
                    .settle_disbursal(
                        disbursal.id,
                        op,
                        obligation,
                        credit_facility.account_ids.facility_account_id,
                    )
                    .await?;
                disbursal
            }
            crate::ApprovalProcessOutcome::Denied(disbursal) => {
                tracing::Span::current().record("already_applied", false);
                let credit_facility = self
                    .credit_facilities
                    .find_by_id_without_audit(disbursal.facility_id) // changed for now
                    .await?;
                self.ledger
                    .cancel_disbursal(
                        disbursal.id,
                        op,
                        disbursal.initiated_tx_id,
                        disbursal.amount,
                        credit_facility.account_ids.facility_account_id,
                    )
                    .await?;
                disbursal
            }
        };

        Ok(disbursal)
    }
}
