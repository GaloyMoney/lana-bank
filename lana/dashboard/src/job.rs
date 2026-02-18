use tracing::{Span, instrument};

use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use crate::repo::DashboardRepo;

pub const DASHBOARD_PROJECTION_JOB: JobType = JobType::new("outbox.dashboard-projection");

pub struct DashboardProjectionHandler {
    repo: DashboardRepo,
}

impl DashboardProjectionHandler {
    pub fn new(repo: &DashboardRepo) -> Self {
        Self { repo: repo.clone() }
    }
}

impl<E> OutboxEventHandler<E> for DashboardProjectionHandler
where
    E: OutboxEventMarker<lana_events::LanaEvent>,
{
    #[instrument(name = "dashboard.projection_job.process_message", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut dashboard = self.repo.load().await?;
        if let Some(payload) = event.as_event::<lana_events::LanaEvent>()
            && dashboard.process_event(event.recorded_at, payload)
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
        }
        self.repo.persist_in_tx(op.tx_mut(), &dashboard).await?;
        Ok(())
    }
}
