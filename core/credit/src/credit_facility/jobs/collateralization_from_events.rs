use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::GovernanceEvent;
use obix::out::{
    EphemeralOutboxEvent, OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent,
};

use job::JobType;

use core_credit_collection::{PublicObligation, PublicPaymentAllocation};
use core_custody::CoreCustodyEvent;
use core_price::{CorePriceEvent, Price};

use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent,
    collateral::Collaterals,
    credit_facility::{
        CreditFacilitiesByCollateralizationRatioCursor, CreditFacilityRepo, CreditFacilityStatus,
    },
    ledger::*,
    primitives::*,
};

pub const CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.credit-facility-collateralization");

pub struct CreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    repo: Arc<CreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

impl<Perms, E> CreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        repo: Arc<CreditFacilityRepo<E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        price: Arc<Price>,
        ledger: Arc<CreditLedger>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            repo,
            collaterals,
            price,
            ledger,
            authz,
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for CreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(name = "core_credit.collateralization_job.process_persistent_message", parent = None, skip(self, _op, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(event @ CoreCreditEvent::FacilityCollateralUpdated { entity }) =
            message.as_event()
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());
            Span::current().record(
                "credit_facility_id",
                tracing::field::display(entity.secured_loan_id),
            );

            self.update_collateralization_from_events(entity.secured_loan_id)
                .await?;
        }

        if let Some(
            event @ (CoreCreditCollectionEvent::ObligationCreated {
                entity: PublicObligation { beneficiary_id, .. },
            }
            | CoreCreditCollectionEvent::PaymentAllocationCreated {
                entity: PublicPaymentAllocation { beneficiary_id, .. },
            }),
        ) = message.as_event()
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());
            Span::current().record(
                "credit_facility_id",
                tracing::field::display(beneficiary_id),
            );

            self.update_collateralization_from_events(*beneficiary_id)
                .await?;
        }

        Ok(())
    }

    #[instrument(name = "core_credit.credit_facility_collateralization_job.process_ephemeral_message", parent = None, skip(self, message), fields(handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn handle_ephemeral(
        &self,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match message.payload.as_event() {
            Some(CorePriceEvent::PriceUpdated { price, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", tracing::field::display(&message.event_type));

                self.update_collateralization_from_price_event(*price)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl<Perms, E> CreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_events",
        skip(self),
        fields(credit_facility_id = %credit_facility_id),
    )]
    #[es_entity::retry_on_concurrent_modification]
    pub(super) async fn update_collateralization_from_events(
        &self,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Display + Copy,
    ) -> Result<(), crate::credit_facility::error::CreditFacilityError> {
        let mut op = self.repo.begin_op().await?;
        // if the pending facility is not collateralized enough to be activated there will be no
        // credit facility to update the collateralization state for
        let Some(mut credit_facility) = self
            .repo
            .maybe_find_by_id_in_op(&mut op, credit_facility_id.into())
            .await?
        else {
            return Ok(());
        };

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
                crate::primitives::COLLATERALIZATION_SYNC,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
            )
            .await?;

        tracing::Span::current().record("credit_facility_id", credit_facility.id.to_string());

        let collateral_account_id = self
            .collaterals
            .find_collateral_ledger_account_ids_in_op(&mut op, credit_facility.collateral_id)
            .await?
            .collateral_account_id;

        let balances = self
            .ledger
            .get_credit_facility_balance_in_op(
                &mut op,
                credit_facility.account_ids,
                collateral_account_id,
            )
            .await?;
        let price = self.price.usd_cents_per_btc().await;

        if credit_facility
            .update_collateralization(price, CVLPct::UPGRADE_BUFFER, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut credit_facility)
                .await?;

            op.commit().await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_price_event",
        skip(self)
    )]
    pub(super) async fn update_collateralization_from_price_event(
        &self,
        price: PriceOfOneBTC,
    ) -> Result<(), crate::credit_facility::error::CreditFacilityError> {
        let mut has_next_page = true;
        let mut after: Option<CreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let credit_facilities =
                self.repo
                    .list_by_collateralization_ratio(
                        es_entity::PaginatedQueryArgs::<
                            CreditFacilitiesByCollateralizationRatioCursor,
                        > {
                            first: 10,
                            after,
                        },
                        es_entity::ListDirection::Ascending,
                    )
                    .await?;
            (after, has_next_page) = (
                credit_facilities.end_cursor,
                credit_facilities.has_next_page,
            );
            let mut op = self.repo.begin_op().await?;
            self.authz
                .audit()
                .record_system_entry_in_op(
                    &mut op,
                    crate::primitives::COLLATERALIZATION_SYNC,
                    CoreCreditObject::all_credit_facilities(),
                    CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
                )
                .await?;

            let mut updated = Vec::new();
            for mut facility in credit_facilities.entities {
                tracing::Span::current().record("credit_facility_id", facility.id.to_string());

                if facility.status() == CreditFacilityStatus::Closed {
                    continue;
                }
                let collateral_account_id = self
                    .collaterals
                    .find_collateral_ledger_account_ids_in_op(&mut op, facility.collateral_id)
                    .await?
                    .collateral_account_id;

                let balances = self
                    .ledger
                    .get_credit_facility_balance_in_op(
                        &mut op,
                        facility.account_ids,
                        collateral_account_id,
                    )
                    .await?;
                if facility
                    .update_collateralization(price, CVLPct::UPGRADE_BUFFER, balances)
                    .did_execute()
                {
                    updated.push(facility);
                }
            }

            let n = self.repo.update_all_in_op(&mut op, &mut updated).await?;

            if n > 0 {
                op.commit().await?;
            } else {
                break;
            }
        }
        Ok(())
    }
}
