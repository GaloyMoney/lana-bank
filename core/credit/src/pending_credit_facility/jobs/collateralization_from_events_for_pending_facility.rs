use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use std::sync::Arc;

use governance::GovernanceEvent;
use obix::out::{
    EphemeralOutboxEvent, OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent,
};

use job::JobType;

use core_custody::CoreCustodyEvent;
use core_price::{CorePriceEvent, Price};

use crate::{
    CoreCreditEvent,
    collateral::CollateralRepo,
    ledger::*,
    pending_credit_facility::{
        PendingCreditFacilitiesByCollateralizationRatioCursor, PendingCreditFacility,
        PendingCreditFacilityError, PendingCreditFacilityRepo,
    },
    primitives::*,
};

pub const PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.pending-credit-facility-collateralization-from-events");

pub struct PendingCreditFacilityCollateralizationFromEventsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    repo: Arc<PendingCreditFacilityRepo<E>>,
    collateral_repo: Arc<CollateralRepo<E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
}

impl<E> PendingCreditFacilityCollateralizationFromEventsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        repo: Arc<PendingCreditFacilityRepo<E>>,
        collateral_repo: Arc<CollateralRepo<E>>,
        price: Arc<Price>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            repo,
            collateral_repo,
            price,
            ledger,
        }
    }
}

impl<E> OutboxEventHandler<E> for PendingCreditFacilityCollateralizationFromEventsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(name = "core_credit.pending_credit_facility_collateralization_job.process_persistent_message", parent = None, skip(self, _op, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, pending_credit_facility_id = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match message.as_event() {
            Some(event @ CoreCreditEvent::FacilityCollateralUpdated { entity }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record(
                    "pending_credit_facility_id",
                    tracing::field::display(entity.secured_loan_id),
                );

                self.update_collateralization_from_events(entity.secured_loan_id)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[instrument(name = "core_credit.pending_credit_facility_collateralization_job.process_ephemeral_message", parent = None, skip(self, message), fields(handled = false, event_type = tracing::field::Empty))]
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

impl<E> PendingCreditFacilityCollateralizationFromEventsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "credit.pending_credit_facility.update_collateralization_from_events",
        skip(self)
    )]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub(super) async fn update_collateralization_from_events(
        &self,
        id: impl Into<PendingCreditFacilityId> + std::fmt::Debug + Copy,
    ) -> Result<PendingCreditFacility, PendingCreditFacilityError> {
        let mut op = self.repo.begin_op().await?;
        let mut pending_facility = self.repo.find_by_id_in_op(&mut op, id.into()).await?;

        tracing::Span::current().record(
            "pending_credit_facility_id",
            pending_facility.id.to_string(),
        );

        let collateral = self
            .collateral_repo
            .find_by_id_in_op(&mut op, pending_facility.collateral_id)
            .await?;
        let collateral_account_id = collateral.account_id();

        let balances = self
            .ledger
            .get_pending_credit_facility_balance(
                pending_facility.account_ids,
                collateral_account_id,
            )
            .await?;

        let price = self.price.usd_cents_per_btc().await;

        if pending_facility
            .update_collateralization(price, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut pending_facility)
                .await?;

            op.commit().await?;
        }
        Ok(pending_facility)
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_price_event",
        skip(self)
    )]
    pub(super) async fn update_collateralization_from_price_event(
        &self,
        price: PriceOfOneBTC,
    ) -> Result<(), PendingCreditFacilityError> {
        let mut has_next_page = true;
        let mut after: Option<PendingCreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let pending_credit_facilities = self
                .repo
                .list_by_collateralization_ratio(
                    es_entity::PaginatedQueryArgs::<
                        PendingCreditFacilitiesByCollateralizationRatioCursor,
                    > {
                        first: 10,
                        after,
                    },
                    Default::default(),
                )
                .await?;
            (after, has_next_page) = (
                pending_credit_facilities.end_cursor,
                pending_credit_facilities.has_next_page,
            );
            let mut op = self.repo.begin_op().await?;

            let mut updated = Vec::new();
            for mut pending_facility in pending_credit_facilities.entities {
                tracing::Span::current().record(
                    "pending_credit_facility_id",
                    pending_facility.id.to_string(),
                );

                if pending_facility.status() == PendingCreditFacilityStatus::Completed {
                    continue;
                }
                let collateral = self
                    .collateral_repo
                    .find_by_id_in_op(&mut op, pending_facility.collateral_id)
                    .await?;
                let balances = self
                    .ledger
                    .get_pending_credit_facility_balance(
                        pending_facility.account_ids,
                        collateral.account_id(),
                    )
                    .await?;
                if pending_facility
                    .update_collateralization(price, balances)
                    .did_execute()
                {
                    updated.push(pending_facility);
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
