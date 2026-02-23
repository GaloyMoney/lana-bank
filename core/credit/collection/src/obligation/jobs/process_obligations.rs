use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{obligation::Obligations, primitives::*, public::CoreCreditCollectionEvent};

const PROCESS_OBLIGATIONS_JOB: JobType = JobType::new("task.process-obligations");

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
}

impl<Perms, E> ProcessObligationsJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(obligations: &Obligations<Perms, E>) -> Self {
        Self {
            obligations: obligations.clone(),
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
        skip(self, _current_job)
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.obligations
            .process_obligations_for_day(self.config.day)
            .await?;
        Ok(JobCompletion::Complete)
    }
}

pub type ProcessObligationsJobSpawner<Perms, E> = JobSpawner<ProcessObligationsJobConfig<Perms, E>>;
