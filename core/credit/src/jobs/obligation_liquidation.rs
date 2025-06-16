use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreCreditEvent, obligation::Obligations, primitives::*};

use super::obligation_defaulted;

#[derive(Clone, Serialize, Deserialize)]
pub struct ObligationLiquidationJobConfig<Perms, E> {
    pub obligation_id: ObligationId,
    pub effective: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for ObligationLiquidationJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = ObligationLiquidationJobInitializer<Perms, E>;
}
pub struct ObligationLiquidationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    obligations: Obligations<Perms, E>,
    jobs: Jobs,
}

impl<Perms, E> ObligationLiquidationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(obligations: &Obligations<Perms, E>, jobs: &Jobs) -> Self {
        Self {
            obligations: obligations.clone(),
            jobs: jobs.clone(),
        }
    }
}

const OBLIGATION_LIQUIDATION_PROCESSING_JOB: JobType =
    JobType::new("obligation-liquidation-processing");
impl<Perms, E> JobInitializer for ObligationLiquidationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        OBLIGATION_LIQUIDATION_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationLiquidationJobRunner::<Perms, E> {
            config: job.config()?,
            obligations: self.obligations.clone(),
            jobs: self.jobs.clone(),
        }))
    }
}

pub struct ObligationLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: ObligationLiquidationJobConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
    jobs: Jobs,
}

#[async_trait]
impl<Perms, E> JobRunner for ObligationLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut db = self.obligations.begin_op().await?;

        let obligation = self
            .obligations
            .start_liquidation_process_in_op(&mut db, self.config.obligation_id)
            .await?;

        if let Some(defaulted_at) = obligation.defaulted_at() {
            self.jobs
                .create_and_spawn_at_in_op(
                    &mut db,
                    JobId::new(),
                    obligation_defaulted::ObligationDefaultedJobConfig::<Perms, E> {
                        obligation_id: obligation.id,
                        effective: defaulted_at.date_naive(),
                        _phantom: std::marker::PhantomData,
                    },
                    defaulted_at,
                )
                .await?;
        }
        Ok(JobCompletion::Complete)
    }
}
