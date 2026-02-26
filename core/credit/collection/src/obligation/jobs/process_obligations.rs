use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use job::*;
use obix::out::OutboxEventMarker;

use super::transition_obligation::{TransitionObligationJobConfig, TransitionObligationJobSpawner};
use crate::{obligation::Obligations, primitives::*, public::CoreCreditCollectionEvent};

const PROCESS_OBLIGATIONS_JOB: JobType = JobType::new("task.process-obligations");
const PAGE_SIZE: i64 = 100;

#[derive(Serialize, Deserialize)]
pub struct ProcessObligationsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub day: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for ProcessObligationsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    fn clone(&self) -> Self {
        Self {
            day: self.day,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct ProcessObligationsJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    obligations: Obligations<Perms, E>,
    transition_spawner: TransitionObligationJobSpawner<Perms, E>,
}

impl<Perms, E> ProcessObligationsJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        obligations: &Obligations<Perms, E>,
        transition_spawner: TransitionObligationJobSpawner<Perms, E>,
    ) -> Self {
        Self {
            obligations: obligations.clone(),
            transition_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for ProcessObligationsJobInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = ProcessObligationsJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        PROCESS_OBLIGATIONS_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ProcessObligationsJobRunner {
            config: job.config()?,
            obligations: self.obligations.clone(),
            transition_spawner: self.transition_spawner.clone(),
        }))
    }
}

pub struct ProcessObligationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: ProcessObligationsJobConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
    transition_spawner: TransitionObligationJobSpawner<Perms, E>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct ProcessObligationsState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, ObligationId)>,
}

#[async_trait]
impl<Perms, E> JobRunner for ProcessObligationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(
        name = "collection.obligation.process_obligations_job",
        skip(self, current_job),
        fields(day = %self.config.day)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<ProcessObligationsState>()?
            .unwrap_or_default();

        loop {
            let rows = self
                .obligations
                .list_ids_needing_transition(self.config.day, state.last_cursor, PAGE_SIZE)
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    JobSpec::new(
                        JobId::new(),
                        TransitionObligationJobConfig {
                            obligation_id: *id,
                            day: self.config.day,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            let mut op = current_job.begin_op().await?;
            self.transition_spawner
                .spawn_all_in_op(&mut op, specs)
                .await?;

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(&mut op, &state)
                .await?;
            op.commit().await?;
        }

        Ok(JobCompletion::Complete)
    }
}

pub type ProcessObligationsJobSpawner<Perms, E> = JobSpawner<ProcessObligationsJobConfig<Perms, E>>;
