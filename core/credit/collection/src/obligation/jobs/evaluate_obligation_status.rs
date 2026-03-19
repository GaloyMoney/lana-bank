use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{obligation::Obligations, primitives::*, public::CoreCreditCollectionEvent};

const EVALUATE_OBLIGATION_STATUS_JOB: JobType = JobType::new("task.evaluate-obligation-status");

#[derive(Serialize, Deserialize)]
pub struct EvaluateObligationStatusConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub obligation_id: ObligationId,
    pub day: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for EvaluateObligationStatusConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    fn clone(&self) -> Self {
        Self {
            obligation_id: self.obligation_id,
            day: self.day,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct EvaluateObligationStatusJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    obligations: Obligations<Perms, E>,
}

impl<Perms, E> EvaluateObligationStatusJobInit<Perms, E>
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

impl<Perms, E> JobInitializer for EvaluateObligationStatusJobInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = EvaluateObligationStatusConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        EVALUATE_OBLIGATION_STATUS_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EvaluateObligationStatusJobRunner {
            config: job.config()?,
            obligations: self.obligations.clone(),
        }))
    }
}

pub struct EvaluateObligationStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: EvaluateObligationStatusConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for EvaluateObligationStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(
        name = "collection.obligation.evaluate_obligation_status",
        skip(self, current_job),
        fields(obligation_id = %self.config.obligation_id, day = %self.config.day)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        self.obligations
            .execute_transition_in_op(&mut op, self.config.obligation_id, self.config.day)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub type EvaluateObligationStatusSpawner<Perms, E> =
    JobSpawner<EvaluateObligationStatusConfig<Perms, E>>;
