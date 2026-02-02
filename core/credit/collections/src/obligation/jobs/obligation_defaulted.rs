use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use authz::PermissionCheck;
use job::*;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreCollectionsEvent,
    ledger::CollectionsLedger,
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

pub(crate) struct ObligationDefaultedInit<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
{
    repo: Arc<ObligationRepo<E>>,
    authz: Arc<Perms>,
    ledger: Arc<CollectionsLedger<L>>,
}

impl<Perms, E, L> ObligationDefaultedInit<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
{
    pub fn new(
        ledger: Arc<CollectionsLedger<L>>,
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
impl<Perms, E, L> JobInitializer for ObligationDefaultedInit<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
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
        Ok(Box::new(ObligationDefaultedJobRunner::<Perms, E, L> {
            config: job.config()?,
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

pub struct ObligationDefaultedJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
{
    config: ObligationDefaultedJobConfig<Perms, E>,
    repo: Arc<ObligationRepo<E>>,
    authz: Arc<Perms>,
    ledger: Arc<CollectionsLedger<L>>,
}

#[async_trait]
impl<Perms, E, L> JobRunner for ObligationDefaultedJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
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

impl<Perms, E, L> ObligationDefaultedJobRunner<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
{
    pub async fn record_defaulted(
        &self,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(), ObligationError> {
        let mut op = self.repo.begin_op().await?;

        let mut obligation = self.repo.find_by_id_in_op(&mut op, id).await?;

        // TODO: Collections authorization to be handled by credit layer
        // self.authz
        //     .audit()
        //     .record_system_entry_in_tx(
        //         &mut op,
        //         CoreCollectionsObject::obligation(id),
        //         CoreCollectionsAction::OBLIGATION_UPDATE_STATUS,
        //     )
        //     .await
        //     .map_err(authz::error::AuthorizationError::from)?;

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
        Ok(())
    }
}

pub type ObligationDefaultedJobSpawner<Perms, E> =
    JobSpawner<ObligationDefaultedJobConfig<Perms, E>>;
