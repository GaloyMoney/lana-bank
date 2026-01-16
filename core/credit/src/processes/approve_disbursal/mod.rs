mod job;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::clock::ClockHandle;
use governance::{
    ApprovalProcessType, Governance, GovernanceAction, GovernanceEvent, GovernanceObject,
};
use tracing::instrument;
use tracing_macros::record_error_severity;

use obix::out::OutboxEventMarker;

use core_accounting::LedgerTransactionInitiator;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;

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
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    disbursals: Arc<Disbursals<Perms, E>>,
    credit_facilities: Arc<CreditFacilities<Perms, E>>,
    governance: Arc<Governance<Perms, E>>,
    ledger: Arc<CreditLedger>,
    clock: ClockHandle,
}

impl<Perms, E> Clone for ApproveDisbursal<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            disbursals: self.disbursals.clone(),
            credit_facilities: self.credit_facilities.clone(),
            governance: self.governance.clone(),
            ledger: self.ledger.clone(),
            clock: self.clock.clone(),
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
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        disbursals: Arc<Disbursals<Perms, E>>,
        credit_facilities: Arc<CreditFacilities<Perms, E>>,
        governance: Arc<Governance<Perms, E>>,
        ledger: Arc<CreditLedger>,
        clock: ClockHandle,
    ) -> Self {
        Self {
            disbursals,
            credit_facilities,
            governance,
            ledger,
            clock,
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "credit_facility.approve_disbursal",
        skip(self),
        fields(already_applied, disbursal_executed)
    )]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub async fn execute_approve_disbursal(
        &self,
        id: impl es_entity::RetryableInto<DisbursalId>,
        approved: bool,
    ) -> Result<Disbursal, CoreCreditError> {
        let mut op = self
            .disbursals
            .begin_op_with_clock(&self.clock)
            .await?
            .with_db_time()
            .await?;

        let disbursal = match self
            .disbursals
            .conclude_approval_process_in_op(&mut op, id.into(), approved)
            .await?
        {
            crate::ApprovalProcessOutcome::AlreadyApplied(disbursal) => {
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
                        &mut op,
                        disbursal.id,
                        disbursal.disbursal_credit_account_id,
                        obligation,
                        credit_facility.account_ids,
                        LedgerTransactionInitiator::System,
                    )
                    .await?;
                op.commit().await?;
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
                        &mut op,
                        disbursal.id,
                        disbursal.initiated_tx_id,
                        disbursal.amount,
                        credit_facility.account_ids,
                        LedgerTransactionInitiator::System,
                    )
                    .await?;
                op.commit().await?;
                disbursal
            }
        };

        Ok(disbursal)
    }
}
