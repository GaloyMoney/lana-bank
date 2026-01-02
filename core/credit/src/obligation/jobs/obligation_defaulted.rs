use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent, ledger::CreditLedger, obligation::ObligationRepo, primitives::*,
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ObligationDefaultedJobConfig<Perms, E> {
    pub obligation_id: ObligationId,
    pub effective: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
pub(crate) struct ObligationDefaultedInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: ObligationRepo<E>,
    ledger: Arc<CreditLedger>,
}

impl<Perms, E> ObligationDefaultedInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(ledger: Arc<CreditLedger>, obligations: &Obligations<Perms, E>) -> Self {
        Self {
            ledger: ledger,
            obligations: obligations.clone(),
        }
    }
}

const OBLIGATION_DEFAULTED_JOB: JobType = JobType::new("task.obligation-defaulted");
impl<Perms, E> JobInitializer for ObligationDefaultedInit<Perms, E>
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
        OBLIGATION_DEFAULTED_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationDefaultedJobRunner::<Perms, E> {
            config: job.config()?,
            obligations: self.obligations.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

pub struct ObligationDefaultedJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: ObligationDefaultedJobConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
    ledger: Arc<CreditLedger>,
}

#[async_trait]
impl<Perms, E> JobRunner for ObligationDefaultedJobRunner<Perms, E>
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
        self.record_defaulted(self.config.obligation_id, self.config.effective)
            .await?;

        Ok(JobCompletion::Complete)
    }
}

impl ObligationDefaultedJobRunner<Perms, E> {
    pub async fn record_defaulted(
        &self,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(), ObligationError> {
        let mut op = self.repo.begin_op().await?;

        let mut obligation = self.repo.find_by_id_in_op(&mut op, id).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op,
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        if let es_entity::Idempotent::Executed(defaulted) =
            obligation.record_defaulted(effective)?
        {
            self.repo.update_in_op(&mut op, &mut obligation).await?;

            self.ledger
                .record_obligation_defaulted(
                    &mut op,
                    defaulted,
                    core_accounting::LedgerTransactionInitiator::System,
                )
                .await?;
            op.commit().await?;
        };
    }
}
