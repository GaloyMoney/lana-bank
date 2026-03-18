use async_trait::async_trait;
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
use obix::out::OutboxEventMarker;

use super::transition_obligation::{TransitionObligationJobConfig, TransitionObligationJobSpawner};
use crate::{obligation::Obligations, primitives::*, public::CoreCreditCollectionEvent};

const PAGE_SIZE: i64 = 100;

pub struct ObligationTransitionProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    jobs: Jobs,
    obligations: Obligations<Perms, E>,
    transition_spawner: TransitionObligationJobSpawner<Perms, E>,
}

impl<Perms, E> ObligationTransitionProcessInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        jobs: &Jobs,
        obligations: &Obligations<Perms, E>,
        transition_spawner: TransitionObligationJobSpawner<Perms, E>,
    ) -> Self {
        Self {
            jobs: jobs.clone(),
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
            jobs: self.jobs.clone(),
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
    jobs: Jobs,
    obligations: Obligations<Perms, E>,
    transition_spawner: TransitionObligationJobSpawner<Perms, E>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ObligationTransitionState {
    #[default]
    Collecting(ObligationTransitionCollectingState),
    Tracking {
        entity_job_ids: Vec<JobId>,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObligationTransitionCollectingState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, ObligationId)>,
    entity_job_ids: Vec<JobId>,
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
                    state.entity_job_ids.push(job_id);
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

        tracing::info!(
            jobs_spawned = state.entity_job_ids.len(),
            "Obligation transition collection complete, transitioning to tracking"
        );

        let new_state = ObligationTransitionState::Tracking {
            entity_job_ids: state.entity_job_ids,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    async fn run_tracking(
        &self,
        _current_job: CurrentJob,
        entity_job_ids: Vec<JobId>,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing::info!(
            total_jobs = entity_job_ids.len(),
            "Awaiting completion of per-entity obligation transition jobs"
        );

        let results = futures::future::try_join_all(
            entity_job_ids
                .iter()
                .map(|id| self.jobs.await_completion(*id)),
        )
        .await?;

        let failed: Vec<_> = entity_job_ids
            .iter()
            .zip(results.iter())
            .filter(|(_, state)| *state != &JobTerminalState::Completed)
            .map(|(id, state)| (*id, state.clone()))
            .collect();

        if !failed.is_empty() {
            tracing::error!(
                ?failed,
                "Some obligation transition entity jobs did not complete successfully"
            );
            return Err(
                format!("{} obligation transition entity jobs failed", failed.len()).into(),
            );
        }

        tracing::info!("All obligation transition entity jobs completed");
        Ok(JobCompletion::Complete)
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
            ObligationTransitionState::Tracking { entity_job_ids } => {
                self.run_tracking(current_job, entity_job_ids).await
            }
        }
    }
}
