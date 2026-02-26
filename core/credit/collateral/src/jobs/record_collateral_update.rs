use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::SystemSubject;
use core_custody::{CUSTODIAN_SYNC, CoreCustodyEvent, WalletId as CustodyWalletId};
use tracing_macros::record_error_severity;

use crate::{CollateralRepo, ledger::CollateralLedger, public::CoreCreditCollateralEvent};

#[derive(Serialize, Deserialize, Clone)]
pub struct RecordCollateralUpdateConfig {
    pub custody_wallet_id: CustodyWalletId,
    pub updated_collateral: money::Satoshis,
    pub effective: chrono::NaiveDate,
}

pub const RECORD_COLLATERAL_UPDATE_COMMAND: JobType =
    JobType::new("command.core-credit.record-collateral-update");

pub struct RecordCollateralUpdateJobInitializer<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, E> RecordCollateralUpdateJobInitializer<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(ledger: Arc<CollateralLedger>, repo: Arc<CollateralRepo<E>>) -> Self {
        Self {
            repo,
            ledger,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, E> JobInitializer for RecordCollateralUpdateJobInitializer<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = RecordCollateralUpdateConfig;

    fn job_type(&self) -> JobType {
        RECORD_COLLATERAL_UPDATE_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RecordCollateralUpdateJobRunner::<S, E> {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            _phantom: std::marker::PhantomData,
        }))
    }
}

pub struct RecordCollateralUpdateJobRunner<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    config: RecordCollateralUpdateConfig,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
    _phantom: std::marker::PhantomData<S>,
}

#[async_trait]
impl<S, E> JobRunner for RecordCollateralUpdateJobRunner<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditCollateralEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_credit.record_collateral_update_job.process_command",
        skip(self, current_job),
        fields(
            custody_wallet_id = %self.config.custody_wallet_id,
            updated_collateral = %self.config.updated_collateral,
            effective = %self.config.effective,
        ),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        let mut collateral = self
            .repo
            .find_by_custody_wallet_id_in_op(&mut op, Some(self.config.custody_wallet_id))
            .await?;

        if let es_entity::Idempotent::Executed(data) = collateral
            .record_collateral_update_via_custodian_sync(
                self.config.updated_collateral,
                self.config.effective,
            )
        {
            self.repo.update_in_op(&mut op, &mut collateral).await?;

            self.ledger
                .update_collateral_amount_in_op(&mut op, data, &S::system(CUSTODIAN_SYNC))
                .await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
