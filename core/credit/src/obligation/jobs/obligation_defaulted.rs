use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent,
    ledger::CreditLedger,
    obligation::{ObligationRepo, error::ObligationError},
    primitives::*,
};

#[derive(Serialize, Deserialize)]
pub struct ObligationDefaultedJobConfig<Perms, E> {
    pub obligation_id: ObligationId,
    pub effective: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for ObligationDefaultedJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            obligation_id: self.obligation_id,
            effective: self.effective,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) struct ObligationDefaultedInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: Arc<ObligationRepo<E>>,
    authz: Arc<Perms>,
    ledger: Arc<CreditLedger>,
}

impl<Perms, E> ObligationDefaultedInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        ledger: Arc<CreditLedger>,
        obligation_repo: Arc<ObligationRepo<E>>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            ledger,
            authz,
            repo: obligation_repo,
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
    type Config = ObligationDefaultedJobConfig<Perms, E>;
    fn job_type(&self) -> JobType {
        OBLIGATION_DEFAULTED_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationDefaultedJobRunner::<Perms, E> {
            config: job.config()?,
            repo: self.repo.clone(),
            authz: self.authz.clone(),
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
    repo: Arc<ObligationRepo<E>>,
    authz: Arc<Perms>,
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

impl<Perms, E> ObligationDefaultedJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub async fn record_defaulted(
        &self,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(), ObligationError> {
        let mut op = self.repo.begin_op().await?;

        let mut obligation = self.repo.find_by_id_in_op(&mut op, id).await?;

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
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
                .record_obligation_defaulted_in_op(
                    &mut op,
                    defaulted,
                    core_accounting::LedgerTransactionInitiator::System,
                )
                .await?;
            op.commit().await?;
        };
        Ok(())
    }
}

pub type ObligationDefaultedJobSpawner<Perms, E> =
    JobSpawner<ObligationDefaultedJobConfig<Perms, E>>;
