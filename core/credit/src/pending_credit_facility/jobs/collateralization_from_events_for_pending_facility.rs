use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::GovernanceEvent;
use obix::out::{
    EphemeralOutboxEvent, OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent,
};

use job::{JobId, JobSpawner, JobType};

use core_credit_collateral::{
    Collaterals, CoreCreditCollateralAction, CoreCreditCollateralObject,
    public::CoreCreditCollateralEvent,
};
use core_credit_collection::{
    CoreCreditCollectionAction, CoreCreditCollectionEvent, CoreCreditCollectionObject,
};
use core_custody::CoreCustodyEvent;
use core_price::{CorePriceEvent, Price};

use crate::{
    CoreCreditEvent,
    ledger::*,
    pending_credit_facility::{
        PendingCreditFacilitiesByCollateralizationRatioCursor, PendingCreditFacilityError,
        PendingCreditFacilityRepo,
    },
    primitives::*,
};

use super::update_pending_collateralization::UpdatePendingCollateralizationConfig;

pub const PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.pending-credit-facility-collateralization-from-events");

pub struct PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    update_pending_collateralization: JobSpawner<UpdatePendingCollateralizationConfig>,
    repo: Arc<PendingCreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
}

impl<Perms, E> PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        update_pending_collateralization: JobSpawner<UpdatePendingCollateralizationConfig>,
        repo: Arc<PendingCreditFacilityRepo<E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        price: Arc<Price>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            update_pending_collateralization,
            repo,
            collaterals,
            price,
            ledger,
        }
    }
}

impl<Perms, E> OutboxEventHandler<E>
    for PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(name = "core_credit.pending_credit_facility_collateralization_job.process_persistent_message", parent = None, skip(self, op, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, pending_credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(event @ CoreCreditCollateralEvent::CollateralUpdated { entity }) =
            message.as_event()
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());
            Span::current().record(
                "pending_credit_facility_id",
                tracing::field::display(entity.secured_loan_id),
            );

            self.update_pending_collateralization
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdatePendingCollateralizationConfig {
                        pending_credit_facility_id: entity.secured_loan_id.into(),
                    },
                    entity.secured_loan_id.to_string(),
                )
                .await?;
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

impl<Perms, E> PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
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
                let collateral_account_id = self
                    .collaterals
                    .collateral_ledger_account_ids_in_op(&mut op, pending_facility.collateral_id)
                    .await?
                    .collateral_account_id;

                let balances = self
                    .ledger
                    .get_pending_credit_facility_balance_in_op(
                        &mut op,
                        pending_facility.account_ids,
                        collateral_account_id,
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
