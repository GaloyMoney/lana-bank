use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use job::JobType;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use super::process_obligations::{ProcessObligationsJobConfig, ProcessObligationsJobSpawner};
use crate::{primitives::*, public::CoreCreditCollectionEvent};

pub const OBLIGATION_END_OF_DAY: JobType = JobType::new("outbox.obligation-end-of-day");

pub struct ObligationEndOfDayHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    spawner: ProcessObligationsJobSpawner<Perms, E>,
}

impl<Perms, E> ObligationEndOfDayHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(spawner: ProcessObligationsJobSpawner<Perms, E>) -> Self {
        Self { spawner }
    }
}

impl<Perms, E> OutboxEventHandler<E> for ObligationEndOfDayHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "obligation.end_of_day.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreTimeEvent::EndOfDay { day, .. }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            self.spawner
                .spawn_in_op(
                    op,
                    job::JobId::new(),
                    ProcessObligationsJobConfig {
                        day: *day,
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
