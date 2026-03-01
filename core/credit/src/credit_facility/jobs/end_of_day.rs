use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use job::JobType;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::collect_facilities_for_accrual::{
    CollectFacilitiesForAccrualJobConfig, CollectFacilitiesForAccrualJobSpawner,
};
use crate::{CoreCreditEvent, primitives::*};

pub const ACCRUAL_END_OF_DAY: JobType = JobType::new("outbox.accrual-end-of-day");

pub struct FacilityEndOfDayHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    collect_facilities_for_accrual: CollectFacilitiesForAccrualJobSpawner<Perms, E>,
}

impl<Perms, E> FacilityEndOfDayHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(
        collect_facilities_for_accrual: CollectFacilitiesForAccrualJobSpawner<Perms, E>,
    ) -> Self {
        Self {
            collect_facilities_for_accrual,
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for FacilityEndOfDayHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "accrual.end_of_day.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
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
                    CollectFacilitiesForAccrualJobConfig {
                        day: *day,
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
