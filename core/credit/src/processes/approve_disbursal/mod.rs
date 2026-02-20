mod job;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{
    ApprovalProcessType, Governance, GovernanceAction, GovernanceEvent, GovernanceObject,
};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::SystemSubject;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditError, CoreCreditEvent, CoreCreditObject, Disbursal, Disbursals,
    collateral::ledger::CollateralLedgerOps, credit_facility::CreditFacilities,
    ledger::CreditLedgerOps, primitives::DisbursalId,
};

use core_credit_collection::CollectionLedgerOps;

pub use job::*;
pub const APPROVE_DISBURSAL_PROCESS: ApprovalProcessType = ApprovalProcessType::new("disbursal");

pub struct ApproveDisbursal<Perms, E, L, CL, ColL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<crate::CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
    ColL: CollectionLedgerOps,
{
    disbursals: Arc<Disbursals<Perms, E, ColL>>,
    credit_facilities: Arc<CreditFacilities<Perms, E, L, CL, ColL>>,
    governance: Arc<Governance<Perms, E>>,
    ledger: Arc<L>,
}

impl<Perms, E, L, CL, ColL> Clone for ApproveDisbursal<Perms, E, L, CL, ColL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<crate::CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
    ColL: CollectionLedgerOps,
{
    fn clone(&self) -> Self {
        Self {
            disbursals: self.disbursals.clone(),
            credit_facilities: self.credit_facilities.clone(),
            governance: self.governance.clone(),
            ledger: self.ledger.clone(),
        }
    }
}

impl<Perms, E, L, CL, ColL> ApproveDisbursal<Perms, E, L, CL, ColL>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<crate::CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<crate::CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<crate::CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
    L: CreditLedgerOps,
    CL: CollateralLedgerOps,
    ColL: CollectionLedgerOps,
{
    pub fn new(
        disbursals: Arc<Disbursals<Perms, E, ColL>>,
        credit_facilities: Arc<CreditFacilities<Perms, E, L, CL, ColL>>,
        governance: Arc<Governance<Perms, E>>,
        ledger: Arc<L>,
    ) -> Self {
        Self {
            disbursals,
            credit_facilities,
            governance,
            ledger,
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
        let mut op = self.disbursals.begin_op().await?.with_db_time().await?;

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
                    .settle_disbursal_in_op(
                        &mut op,
                        disbursal.id,
                        disbursal.disbursal_credit_account_id,
                        obligation,
                        credit_facility.account_ids,
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                            crate::primitives::DISBURSAL_APPROVAL,
                        ),
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
                    .cancel_disbursal_in_op(
                        &mut op,
                        disbursal.id,
                        disbursal.initiated_tx_id,
                        disbursal.amount,
                        credit_facility.account_ids,
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                            crate::primitives::DISBURSAL_APPROVAL,
                        ),
                    )
                    .await?;
                op.commit().await?;
                disbursal
            }
        };

        Ok(disbursal)
    }
}
