use tracing::{Span, instrument};

use core_time_events::CoreTimeEvent;
use job::JobType;
use obix::out::{OutboxEventHandler, OutboxEventMarker};

use super::process_facility_maturities::{
    ProcessFacilityMaturitiesJobConfig, ProcessFacilityMaturitiesJobSpawner,
};
use crate::CoreCreditEvent;

pub const CREDIT_FACILITY_MATURITY_END_OF_DAY: JobType =
    JobType::new("outbox.credit-facility-maturity-end-of-day");

pub struct CreditFacilityMaturityEndOfDayHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    process_maturities: ProcessFacilityMaturitiesJobSpawner<E>,
}

impl<E> CreditFacilityMaturityEndOfDayHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(process_maturities: ProcessFacilityMaturitiesJobSpawner<E>) -> Self {
        Self { process_maturities }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityMaturityEndOfDayHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "credit_facility.maturity_end_of_day.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &obix::out::PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreTimeEvent::EndOfDay { day, .. }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.process_maturities
                .spawn_in_op(
                    op,
                    job::JobId::new(),
                    ProcessFacilityMaturitiesJobConfig {
                        day: *day,
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
