use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreCreditCollectionsEvent,
    ledger::CollectionsLedger,
    obligation::{ObligationError, ObligationRepo},
    primitives::*,
};

use super::obligation_defaulted::{ObligationDefaultedJobConfig, ObligationDefaultedJobSpawner};

#[derive(Serialize, Deserialize)]
pub struct ObligationOverdueJobConfig<Perms, E> {
    pub obligation_id: ObligationId,
    pub effective: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for ObligationOverdueJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            obligation_id: self.obligation_id,
            effective: self.effective,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct ObligationOverdueInit<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    repo: Arc<ObligationRepo<E>>,
    ledger: Arc<L>,
    authz: Arc<Perms>,
    obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
}

impl<Perms, E, L> ObligationOverdueInit<Perms, E, L>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionsObject>,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    pub fn new(
        ledger: Arc<L>,
        obligation_repo: Arc<ObligationRepo<E>>,
        authz: Arc<Perms>,
        obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
    ) -> Self {
        Self {
            ledger,
            authz,
            repo: obligation_repo,
            obligation_defaulted_job_spawner,
        }
    }
}

const OBLIGATION_OVERDUE_JOB: JobType = JobType::new("task.obligation-overdue");
impl<Perms, E, L> JobInitializer for ObligationOverdueInit<Perms, E, L>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionsObject>,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    type Config = ObligationOverdueJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        OBLIGATION_OVERDUE_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationOverdueJobRunner::<Perms, E, L> {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            obligation_defaulted_job_spawner: self.obligation_defaulted_job_spawner.clone(),
        }))
    }
}

pub struct ObligationOverdueJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    config: ObligationOverdueJobConfig<Perms, E>,
    repo: Arc<ObligationRepo<E>>,
    ledger: Arc<L>,
    authz: Arc<Perms>,
    obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
}

#[async_trait]
impl<Perms, E, L> JobRunner for ObligationOverdueJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionsObject>,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.record_overdue(self.config.obligation_id, self.config.effective)
            .await?;

        Ok(JobCompletion::Complete)
    }
}

impl<Perms, E, L> ObligationOverdueJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionsObject>,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    pub async fn record_overdue(
        &self,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(), ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        let mut op = self.repo.begin_op().await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
                CoreCreditCollectionsObject::obligation(id),
                CoreCreditCollectionsAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        if let es_entity::Idempotent::Executed(data) = obligation.record_overdue(effective)? {
            self.repo.update_in_op(&mut op, &mut obligation).await?;

            if let Some(defaulted_at) = obligation.defaulted_at() {
                self.obligation_defaulted_job_spawner
                    .spawn_at_in_op(
                        &mut op,
                        JobId::new(),
                        ObligationDefaultedJobConfig::<Perms, E> {
                            obligation_id: obligation.id,
                            effective: defaulted_at.date_naive(),
                            _phantom: std::marker::PhantomData,
                        },
                        defaulted_at,
                    )
                    .await?;
            }

            self.ledger
                .record_obligation_overdue(
                    &mut op,
                    data,
                    core_accounting::LedgerTransactionInitiator::System,
                )
                .await?;

            op.commit().await?;
        }
        Ok(())
    }
}

pub type ObligationOverdueJobSpawner<Perms, E> = JobSpawner<ObligationOverdueJobConfig<Perms, E>>;
