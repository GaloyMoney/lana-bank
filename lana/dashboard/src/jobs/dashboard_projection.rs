use tracing::{Span, instrument};

use lana_events::*;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::update_dashboard::{UpdateDashboardConfig, UpdateDashboardUpdate};

pub const DASHBOARD_PROJECTION_JOB: JobType = JobType::new("outbox.dashboard-projection");

pub struct DashboardProjectionHandler {
    update_dashboard: JobSpawner<UpdateDashboardConfig>,
}

impl DashboardProjectionHandler {
    pub fn new(update_dashboard: JobSpawner<UpdateDashboardConfig>) -> Self {
        Self { update_dashboard }
    }
}

impl<E> OutboxEventHandler<E> for DashboardProjectionHandler
where
    E: OutboxEventMarker<LanaEvent>,
{
    #[instrument(name = "dashboard.projection_job.process_message", parent = None, skip_all, fields(seq = %event.sequence, handled = false))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let update = match event.as_event::<LanaEvent>() {
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityProposalCreated { .. })) => {
                Some(UpdateDashboardUpdate::FacilityProposalCreated {
                    recorded_at: event.recorded_at,
                })
            }
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { .. })) => {
                Some(UpdateDashboardUpdate::FacilityActivated {
                    recorded_at: event.recorded_at,
                })
            }
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityCompleted { .. })) => {
                Some(UpdateDashboardUpdate::FacilityCompleted {
                    recorded_at: event.recorded_at,
                })
            }
            Some(LanaEvent::Credit(CoreCreditEvent::DisbursalSettled { entity })) => {
                Some(UpdateDashboardUpdate::DisbursalSettled {
                    recorded_at: event.recorded_at,
                    amount: entity.amount,
                })
            }
            Some(LanaEvent::CreditCollection(
                CoreCreditCollectionEvent::PaymentAllocationCreated { entity },
            )) => Some(UpdateDashboardUpdate::PaymentAllocationCreated {
                recorded_at: event.recorded_at,
                amount: entity.amount,
                obligation_type: entity.obligation_type,
            }),
            Some(LanaEvent::CreditCollateral(CoreCreditCollateralEvent::CollateralUpdated {
                entity,
            })) => {
                let adjustment = entity
                    .adjustment
                    .as_ref()
                    .expect("adjustment must be set for CollateralUpdated");
                Some(UpdateDashboardUpdate::CollateralUpdated {
                    recorded_at: event.recorded_at,
                    direction: adjustment.direction,
                    abs_diff: adjustment.abs_diff,
                })
            }
            _ => None,
        };

        if let Some(update) = update {
            event.inject_trace_parent();
            Span::current().record("handled", true);

            self.update_dashboard
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    UpdateDashboardConfig {
                        update,
                        trace_context: Some(tracing_utils::persistence::extract()),
                    },
                    "dashboard".to_string(),
                )
                .await?;
        }

        Ok(())
    }
}
