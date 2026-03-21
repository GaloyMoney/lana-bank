use tracing::{Span, instrument};

use std::sync::Arc;

use authz::PermissionCheck;
use governance::GovernanceEvent;
use obix::out::{
    EphemeralOutboxEvent, OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent,
};

use job::{JobId, JobSpawner, JobType};

use core_credit_collateral::CoreCreditCollateralEvent;
use core_credit_collection::CoreCreditCollectionEvent;
use core_custody::CoreCustodyEvent;
use core_price::CorePriceEvent;

use crate::{
    CoreCreditEvent,
    pending_credit_facility::{PendingCreditFacilityId, PendingCreditFacilityRepo},
};

use super::update_pending_collateralization::UpdatePendingCollateralizationConfig;

pub const PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.pending-credit-facility-collateralization-from-events");

const PAGE_SIZE: i64 = 100;

pub struct PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    update_pending_collateralization: JobSpawner<UpdatePendingCollateralizationConfig>,
    repo: Arc<PendingCreditFacilityRepo<E>>,
    _perms: std::marker::PhantomData<Perms>,
}

impl<Perms, E> PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        update_pending_collateralization: JobSpawner<UpdatePendingCollateralizationConfig>,
        repo: Arc<PendingCreditFacilityRepo<E>>,
    ) -> Self {
        Self {
            update_pending_collateralization,
            repo,
            _perms: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> OutboxEventHandler<E>
    for PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
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
            Some(CorePriceEvent::PriceUpdated { .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", tracing::field::display(&message.event_type));

                self.spawn_pending_collateralization_updates().await?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl<Perms, E> PendingCreditFacilityCollateralizationFromEventsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(
        name = "credit.pending_credit_facility.spawn_collateralization_updates_from_price_event",
        skip(self)
    )]
    async fn spawn_pending_collateralization_updates(
        &self,
    ) -> Result<(), crate::pending_credit_facility::error::PendingCreditFacilityError> {
        let mut last_cursor: Option<(chrono::DateTime<chrono::Utc>, PendingCreditFacilityId)> =
            None;

        loop {
            let rows = self
                .repo
                .list_non_completed_pending_facility_ids(last_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let mut op = self.repo.begin_op().await?;
            for (id, _) in &rows {
                self.update_pending_collateralization
                    .spawn_with_queue_id_in_op(
                        &mut op,
                        JobId::new(),
                        UpdatePendingCollateralizationConfig {
                            pending_credit_facility_id: *id,
                        },
                        id.to_string(),
                    )
                    .await?;
            }

            last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            op.commit().await?;
        }

        Ok(())
    }
}
