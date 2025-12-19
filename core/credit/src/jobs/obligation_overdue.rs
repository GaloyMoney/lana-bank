use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{event::CoreCreditEvent, ledger::CreditLedger, obligation::Obligations, primitives::*};

use super::obligation_defaulted;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ObligationOverdueJobConfig<Perms, E> {
    pub obligation_id: ObligationId,
    pub effective: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for ObligationOverdueJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = ObligationOverdueInit<Perms, E>;
}
pub struct ObligationOverdueInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    obligations: Obligations<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
}

impl<Perms, E> ObligationOverdueInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(ledger: &CreditLedger, obligations: &Obligations<Perms, E>, jobs: &Jobs) -> Self {
        Self {
            ledger: ledger.clone(),
            obligations: obligations.clone(),
            jobs: jobs.clone(),
        }
    }
}

const OBLIGATION_OVERDUE_JOB: JobType = JobType::new("task.obligation-overdue");
impl<Perms, E> JobInitializer for ObligationOverdueInit<Perms, E>
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
        OBLIGATION_OVERDUE_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationOverdueJobRunner::<Perms, E> {
            config: job.config()?,
            obligations: self.obligations.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
        }))
    }
}

pub struct ObligationOverdueJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: ObligationOverdueJobConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
}

#[async_trait]
impl<Perms, E> JobRunner for ObligationOverdueJobRunner<Perms, E>
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
        let (obligation, overdue_data) = self
            .obligations
            .record_overdue_in_op(&mut db, self.config.obligation_id, self.config.effective)
            .await?;

        let overdue = if let Some(overdue) = overdue_data {
            overdue
        } else {
            return Ok(JobCompletion::CompleteWithOp(db));
        };

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

        self.ledger
            .record_obligation_overdue(&mut db, overdue)
            .await?;

        Ok(JobCompletion::CompleteWithOp(db))
    }
}
