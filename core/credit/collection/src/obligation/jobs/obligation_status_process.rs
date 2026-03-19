use std::collections::HashSet;

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use core_time_events::obligation_status_process::{
    OBLIGATION_STATUS_PROCESS_JOB_TYPE, ObligationStatusProcessConfig,
};
use job::{error::JobError, *};
use obix::out::{Outbox, OutboxEventMarker};

use super::evaluate_obligation_status::{
    EvaluateObligationStatusConfig, EvaluateObligationStatusSpawner,
};
use crate::{
    obligation::Obligations,
    primitives::*,
    public::{CoreCreditCollectionEvent, PublicObligation},
};

const PAGE_SIZE: i64 = 100;

pub struct ObligationStatusProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    outbox: Outbox<E>,
    obligations: Obligations<Perms, E>,
    evaluate_spawner: EvaluateObligationStatusSpawner<Perms, E>,
}

impl<Perms, E> ObligationStatusProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        obligations: &Obligations<Perms, E>,
        evaluate_spawner: EvaluateObligationStatusSpawner<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            obligations: obligations.clone(),
            evaluate_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for ObligationStatusProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = ObligationStatusProcessConfig;

    fn job_type(&self) -> JobType {
        OBLIGATION_STATUS_PROCESS_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationStatusProcessRunner {
            config: job.config()?,
            outbox: self.outbox.clone(),
            obligations: self.obligations.clone(),
            evaluate_spawner: self.evaluate_spawner.clone(),
        }))
    }
}

struct ObligationStatusProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: ObligationStatusProcessConfig,
    outbox: Outbox<E>,
    obligations: Obligations<Perms, E>,
    evaluate_spawner: EvaluateObligationStatusSpawner<Perms, E>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ObligationStatusState {
    #[default]
    SpawningStatusJobs(SpawningStatusJobsState),
    AwaitingStatusUpdates {
        pending: HashSet<ObligationId>,
        start_sequence: i64,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpawningStatusJobsState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, ObligationId)>,
    pending: HashSet<ObligationId>,
    /// Captured once on first entry; reused on crash-restart to avoid
    /// missing events from children that completed before the restart.
    start_sequence: Option<i64>,
}

impl<Perms, E> ObligationStatusProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    /// Step 1: Capture the outbox sequence, then query obligations needing
    /// status evaluation (paginated) and spawn a per-obligation status job for
    /// each. Transitions to AwaitingStatusUpdates when all pages are processed.
    async fn spawn_status_jobs(
        &self,
        mut current_job: CurrentJob,
        mut state: SpawningStatusJobsState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Capture sequence ONCE on first entry; reuse the persisted value on
        // crash-restart so we never miss events from fast-finishing children.
        let start_sequence = match state.start_sequence {
            Some(seq) => seq,
            None => {
                let seq = self.outbox.current_sequence().await?;
                state.start_sequence = Some(seq);
                current_job
                    .update_execution_state(&ObligationStatusState::SpawningStatusJobs(
                        state.clone(),
                    ))
                    .await?;
                seq
            }
        };

        loop {
            let mut op = current_job.begin_op().await?;

            let rows = self
                .obligations
                .list_ids_needing_transition_in_op(
                    &mut op,
                    self.config.date,
                    state.last_cursor,
                    PAGE_SIZE,
                )
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    let job_id = JobId::new();
                    state.pending.insert(*id);
                    JobSpec::new(
                        job_id,
                        EvaluateObligationStatusConfig {
                            obligation_id: *id,
                            day: self.config.date,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            match self.evaluate_spawner.spawn_all_in_op(&mut op, specs).await {
                Ok(_) | Err(JobError::DuplicateId(_)) => {}
                Err(e) => return Err(e.into()),
            }

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(
                    &mut op,
                    &ObligationStatusState::SpawningStatusJobs(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        tracing::info!(
            entities = state.pending.len(),
            start_sequence,
            "Obligation status spawning complete, transitioning to awaiting"
        );

        let new_state = ObligationStatusState::AwaitingStatusUpdates {
            pending: state.pending,
            start_sequence,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    fn extract_obligation_completion(event: &CoreCreditCollectionEvent) -> Option<ObligationId> {
        match event {
            CoreCreditCollectionEvent::ObligationDue {
                entity: PublicObligation { id, .. },
            }
            | CoreCreditCollectionEvent::ObligationOverdue {
                entity: PublicObligation { id, .. },
            }
            | CoreCreditCollectionEvent::ObligationDefaulted {
                entity: PublicObligation { id, .. },
            } => Some(*id),
            _ => None,
        }
    }

    /// Step 2: Stream outbox events from the saved sequence, matching
    /// obligation status completion events. Removes completed obligations
    /// from the pending set and checkpoints on each match. Completes when all
    /// obligations have been evaluated.
    async fn await_status_events(
        &self,
        mut current_job: CurrentJob,
        mut pending: HashSet<ObligationId>,
        mut start_sequence: i64,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if pending.is_empty() {
            tracing::info!("No obligations to track, completing immediately");
            return Ok(JobCompletion::Complete);
        }

        tracing::info!(
            remaining = pending.len(),
            start_sequence,
            "Streaming outbox events for obligation status completion"
        );

        let mut stream = self.outbox.listen_persisted(Some(start_sequence));

        loop {
            tokio::select! {
                Some(event) = stream.next() => {
                    let matched_id = event.payload.as_ref()
                        .and_then(|p| p.as_event::<CoreCreditCollectionEvent>())
                        .and_then(Self::extract_obligation_completion);

                    if let Some(obligation_id) = matched_id {
                        if pending.remove(&obligation_id) {
                            start_sequence = event.sequence;
                            let state = ObligationStatusState::AwaitingStatusUpdates {
                                pending: pending.clone(),
                                start_sequence,
                            };
                            current_job.update_execution_state(&state).await?;
                        }
                    }
                    if pending.is_empty() {
                        tracing::info!("All obligation status updates completed");
                        return Ok(JobCompletion::Complete);
                    }
                }
                _ = current_job.shutdown_requested() => {
                    let state = ObligationStatusState::AwaitingStatusUpdates {
                        pending,
                        start_sequence,
                    };
                    current_job.update_execution_state(&state).await?;
                    tracing::info!("Shutdown requested, rescheduling obligation status tracking");
                    return Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO));
                }
            }
        }
    }
}

#[async_trait]
impl<Perms, E> JobRunner for ObligationStatusProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.obligation-status-process.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<ObligationStatusState>()?
            .unwrap_or_default();

        match state {
            ObligationStatusState::SpawningStatusJobs(spawning) => {
                self.spawn_status_jobs(current_job, spawning).await
            }
            ObligationStatusState::AwaitingStatusUpdates {
                pending,
                start_sequence,
            } => {
                self.await_status_events(current_job, pending, start_sequence)
                    .await
            }
        }
    }
}
