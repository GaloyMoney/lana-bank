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
use super::obligation_overdue::{ObligationOverdueJobConfig, ObligationOverdueJobSpawner};

#[derive(Serialize, Deserialize)]
pub struct ObligationDueJobConfig<Perms, E> {
    pub obligation_id: ObligationId,
    pub effective: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for ObligationDueJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            obligation_id: self.obligation_id,
            effective: self.effective,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) struct ObligationDueInit<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    repo: Arc<ObligationRepo<E>>,
    ledger: Arc<L>,
    authz: Arc<Perms>,
    obligation_overdue_job_spawner: ObligationOverdueJobSpawner<Perms, E>,
    obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
}

impl<Perms, E, L> ObligationDueInit<Perms, E, L>
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
        obligation_overdue_job_spawner: ObligationOverdueJobSpawner<Perms, E>,
        obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
    ) -> Self {
        Self {
            ledger,
            authz,
            repo: obligation_repo,
            obligation_overdue_job_spawner,
            obligation_defaulted_job_spawner,
        }
    }
}

const OBLIGATION_DUE_JOB: JobType = JobType::new("task.obligation-due");
impl<Perms, E, L> JobInitializer for ObligationDueInit<Perms, E, L>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionsObject>,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    type Config = ObligationDueJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        OBLIGATION_DUE_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationDueJobRunner::<Perms, E, L> {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            obligation_overdue_job_spawner: self.obligation_overdue_job_spawner.clone(),
            obligation_defaulted_job_spawner: self.obligation_defaulted_job_spawner.clone(),
        }))
    }
}

pub struct ObligationDueJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    config: ObligationDueJobConfig<Perms, E>,
    repo: Arc<ObligationRepo<E>>,
    ledger: Arc<L>,
    authz: Arc<Perms>,
    obligation_overdue_job_spawner: ObligationOverdueJobSpawner<Perms, E>,
    obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
}

#[async_trait]
impl<Perms, E, L> JobRunner for ObligationDueJobRunner<Perms, E, L>
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
        self.record_due(self.config.obligation_id, self.config.effective)
            .await?;

        Ok(JobCompletion::Complete)
    }
}

impl<Perms, E, L> ObligationDueJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionsAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionsObject>,
    E: OutboxEventMarker<CoreCreditCollectionsEvent>,
    L: CollectionsLedger,
{
    pub async fn record_due(
        &self,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(), ObligationError> {
        let mut op = self.repo.begin_op().await?;

        let mut obligation = self.repo.find_by_id_in_op(&mut op, id).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                &mut op,
                CoreCreditCollectionsObject::obligation(id),
                CoreCreditCollectionsAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        if let es_entity::Idempotent::Executed(due_data) = obligation.record_due(effective) {
            self.repo.update_in_op(&mut op, &mut obligation).await?;

            if let Some(overdue_at) = obligation.overdue_at() {
                self.obligation_overdue_job_spawner
                    .spawn_at_in_op(
                        &mut op,
                        JobId::new(),
                        ObligationOverdueJobConfig::<Perms, E> {
                            obligation_id: obligation.id,
                            effective: overdue_at.date_naive(),
                            _phantom: std::marker::PhantomData,
                        },
                        overdue_at,
                    )
                    .await?;
            } else if let Some(defaulted_at) = obligation.defaulted_at() {
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
                .record_obligation_due(
                    &mut op,
                    due_data,
                    core_accounting::LedgerTransactionInitiator::System,
                )
                .await?;

            op.commit().await?;
        }
        Ok(())
    }
}

pub type ObligationDueJobSpawner<Perms, E> = JobSpawner<ObligationDueJobConfig<Perms, E>>;
