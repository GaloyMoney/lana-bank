use tracing::{Span, instrument};

use core_time_events::CoreTimeEvent;
use job::JobType;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::collect_facilities_for_accrual::{
    CollectFacilitiesForAccrualJobConfig, CollectFacilitiesForAccrualJobSpawner,
};
use super::process_facility_maturities::{
    ProcessFacilityMaturitiesJobConfig, ProcessFacilityMaturitiesJobSpawner,
};

pub const FACILITY_END_OF_DAY: JobType = JobType::new("outbox.facility-end-of-day");

pub struct FacilityEndOfDayHandler {
    collect_facilities_for_accrual: CollectFacilitiesForAccrualJobSpawner,
    process_maturities: ProcessFacilityMaturitiesJobSpawner,
}

impl FacilityEndOfDayHandler {
    pub fn new(
        collect_facilities_for_accrual: CollectFacilitiesForAccrualJobSpawner,
        process_maturities: ProcessFacilityMaturitiesJobSpawner,
    ) -> Self {
        Self {
            collect_facilities_for_accrual,
            process_maturities,
        }
    }
}

impl<E> OutboxEventHandler<E> for FacilityEndOfDayHandler
where
    E: OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "facility.end_of_day.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreTimeEvent::EndOfDay { day, .. }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.collect_facilities_for_accrual
                .spawn_in_op(
                    op,
                    job::JobId::new(),
                    CollectFacilitiesForAccrualJobConfig { day: *day },
                )
                .await?;

            self.process_maturities
                .spawn_in_op(
                    op,
                    job::JobId::new(),
                    ProcessFacilityMaturitiesJobConfig { day: *day },
                )
                .await?;
        }
        Ok(())
    }
}
