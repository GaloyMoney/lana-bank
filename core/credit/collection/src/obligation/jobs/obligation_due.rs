use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use audit::{AuditSvc, SystemActor};
use authz::PermissionCheck;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{
    ledger::CollectionLedger,
    obligation::{ObligationError, ObligationRepo},
    primitives::*,
    public::CoreCreditCollectionEvent,
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

pub(crate) struct ObligationDueInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    repo: Arc<ObligationRepo<E>>,
    ledger: Arc<CollectionLedger>,
    authz: Arc<Perms>,
    obligation_overdue_job_spawner: ObligationOverdueJobSpawner<Perms, E>,
    obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
}

impl<Perms, E> ObligationDueInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        ledger: Arc<CollectionLedger>,
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
impl<Perms, E> JobInitializer for ObligationDueInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
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
        Ok(Box::new(ObligationDueJobRunner::<Perms, E> {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            obligation_overdue_job_spawner: self.obligation_overdue_job_spawner.clone(),
            obligation_defaulted_job_spawner: self.obligation_defaulted_job_spawner.clone(),
        }))
    }
}

pub struct ObligationDueJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: ObligationDueJobConfig<Perms, E>,
    repo: Arc<ObligationRepo<E>>,
    ledger: Arc<CollectionLedger>,
    authz: Arc<Perms>,
    obligation_overdue_job_spawner: ObligationOverdueJobSpawner<Perms, E>,
    obligation_defaulted_job_spawner: ObligationDefaultedJobSpawner<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ObligationDueJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
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

impl<Perms, E> ObligationDueJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
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
            .record_system_entry_in_op(
                &mut op,
                SystemActor::ObligationSync,
                CoreCreditCollectionObject::obligation(id),
                CoreCreditCollectionAction::OBLIGATION_UPDATE_STATUS,
            )
            .await?;

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
                .record_obligation_due_in_op(
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
