use std::collections::HashSet;

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use core_time_events::obligation_transition_process::{
    OBLIGATION_TRANSITION_PROCESS_JOB_TYPE, ObligationTransitionProcessConfig,
};
use job::{error::JobError, *};
use obix::out::{Outbox, OutboxEventMarker};

use super::transition_obligation::{TransitionObligationJobConfig, TransitionObligationJobSpawner};
use crate::{
    obligation::Obligations,
    primitives::*,
    public::{CoreCreditCollectionEvent, PublicObligation},
};

const PAGE_SIZE: i64 = 100;

pub struct ObligationTransitionProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    outbox: Outbox<E>,
    obligations: Obligations<Perms, E>,
    transition_spawner: TransitionObligationJobSpawner<Perms, E>,
}

impl<Perms, E> ObligationTransitionProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        obligations: &Obligations<Perms, E>,
        transition_spawner: TransitionObligationJobSpawner<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            obligations: obligations.clone(),
            transition_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for ObligationTransitionProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = ObligationTransitionProcessConfig;

    fn job_type(&self) -> JobType {
        OBLIGATION_TRANSITION_PROCESS_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationTransitionProcessRunner {
            config: job.config()?,
            outbox: self.outbox.clone(),
            obligations: self.obligations.clone(),
            transition_spawner: self.transition_spawner.clone(),
        }))
    }
}

struct ObligationTransitionProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: ObligationTransitionProcessConfig,
    outbox: Outbox<E>,
    obligations: Obligations<Perms, E>,
    transition_spawner: TransitionObligationJobSpawner<Perms, E>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ObligationTransitionState {
    #[default]
    SpawningTransitionJobs(SpawningTransitionJobsState),
    AwaitingTransitions {
        pending: HashSet<ObligationId>,
        start_sequence: i64,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpawningTransitionJobsState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, ObligationId)>,
    pending: HashSet<ObligationId>,
    /// Captured once on first entry; reused on crash-restart to avoid
    /// missing events from children that completed before the restart.
    start_sequence: Option<i64>,
}

impl<Perms, E> ObligationTransitionProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    /// Step 1: Capture the outbox sequence, then query obligations needing
    /// transition (paginated) and spawn a per-obligation transition job for
    /// each. Transitions to AwaitingTransitions when all pages are processed.
    async fn spawn_transition_jobs(
        &self,
        mut current_job: CurrentJob,
        mut state: SpawningTransitionJobsState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Capture sequence ONCE on first entry; reuse the persisted value on
        // crash-restart so we never miss events from fast-finishing children.
        let start_sequence = match state.start_sequence {
            Some(seq) => seq,
            None => {
                let seq = self.outbox.current_sequence().await?;
                state.start_sequence = Some(seq);
                current_job
                    .update_execution_state(&ObligationTransitionState::SpawningTransitionJobs(
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
                    let job_id = core_time_events::eod_entity_id(
                        &self.config.date,
                        "obligation-transition",
                        &(*id).into(),
                    );
                    state.pending.insert(*id);
                    JobSpec::new(
                        job_id,
                        TransitionObligationJobConfig {
                            obligation_id: *id,
                            day: self.config.date,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            match self
                .transition_spawner
                .spawn_all_in_op(&mut op, specs)
                .await
            {
                Ok(_) | Err(JobError::DuplicateId(_)) => {}
                Err(e) => return Err(e.into()),
            }

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(
                    &mut op,
                    &ObligationTransitionState::SpawningTransitionJobs(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        tracing::info!(
            entities = state.pending.len(),
            start_sequence,
            "Obligation transition spawning complete, transitioning to awaiting"
        );

        let new_state = ObligationTransitionState::AwaitingTransitions {
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
    /// obligation transition completion events. Removes completed obligations
    /// from the pending set and checkpoints on each match. Completes when all
    /// obligations have transitioned.
    async fn await_transition_events(
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
            "Streaming outbox events for obligation transition completion"
        );

        let mut stream = self.outbox.listen_persisted(Some(start_sequence));

        loop {
            tokio::select! {
                Some(event) = stream.next() => {
                    if let Some(payload) = event.payload.as_ref() {
                        if let Some(collection_event) = payload.as_event::<CoreCreditCollectionEvent>() {
                            if let Some(obligation_id) = Self::extract_obligation_completion(collection_event) {
                                if pending.remove(&obligation_id) {
                                    start_sequence = event.sequence;
                                    let state = ObligationTransitionState::AwaitingTransitions {
                                        pending: pending.clone(),
                                        start_sequence,
                                    };
                                    current_job.update_execution_state(&state).await?;
                                }
                            }
                        }
                    }
                    if pending.is_empty() {
                        tracing::info!("All obligation transitions completed");
                        return Ok(JobCompletion::Complete);
                    }
                }
                _ = current_job.shutdown_requested() => {
                    let state = ObligationTransitionState::AwaitingTransitions {
                        pending,
                        start_sequence,
                    };
                    current_job.update_execution_state(&state).await?;
                    tracing::info!("Shutdown requested, rescheduling obligation transition tracking");
                    return Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO));
                }
            }
        }
    }
}

#[async_trait]
impl<Perms, E> JobRunner for ObligationTransitionProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.obligation-transition-process.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<ObligationTransitionState>()?
            .unwrap_or_default();

        match state {
            ObligationTransitionState::SpawningTransitionJobs(spawning) => {
                self.spawn_transition_jobs(current_job, spawning).await
            }
            ObligationTransitionState::AwaitingTransitions {
                pending,
                start_sequence,
            } => {
                self.await_transition_events(current_job, pending, start_sequence)
                    .await
            }
        }
    }
}
