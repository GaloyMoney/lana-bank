use std::collections::HashSet;

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_eod::obligation_transition_process::{
    OBLIGATION_TRANSITION_PROCESS_JOB_TYPE, ObligationTransitionProcessConfig,
};
use core_time_events::CoreTimeEvent;
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
    Collecting(ObligationTransitionCollectingState),
    Tracking {
        pending: HashSet<ObligationId>,
        start_sequence: i64,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObligationTransitionCollectingState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, ObligationId)>,
    pending: HashSet<ObligationId>,
}

impl<Perms, E> ObligationTransitionProcessRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    async fn run_collecting(
        &self,
        mut current_job: CurrentJob,
        mut state: ObligationTransitionCollectingState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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
                    let job_id = core_eod::eod_entity_id(
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
                    &ObligationTransitionState::Collecting(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        let start_sequence = self.outbox.current_sequence().await?;

        tracing::info!(
            entities = state.pending.len(),
            start_sequence,
            "Obligation transition collection complete, transitioning to tracking"
        );

        let new_state = ObligationTransitionState::Tracking {
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

    async fn run_tracking(
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
                                    let state = ObligationTransitionState::Tracking {
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
            ObligationTransitionState::Collecting(collecting) => {
                self.run_collecting(current_job, collecting).await
            }
            ObligationTransitionState::Tracking {
                pending,
                start_sequence,
            } => {
                self.run_tracking(current_job, pending, start_sequence)
                    .await
            }
        }
    }
}
