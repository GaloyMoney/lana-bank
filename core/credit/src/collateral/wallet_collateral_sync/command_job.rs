use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::SystemSubject;
use core_custody::CoreCustodyEvent;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditEvent,
    collateral::{CollateralRepo, ledger::CollateralLedger},
    primitives::CustodyWalletId,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct RecordCollateralUpdateViaCustodianSyncConfig {
    pub custody_wallet_id: CustodyWalletId,
    pub updated_collateral: money::Satoshis,
    pub effective: chrono::NaiveDate,
}

pub const RECORD_COLLATERAL_UPDATE_VIA_CUSTODIAN_SYNC_COMMAND: JobType =
    JobType::new("command.core-credit.record-collateral-update-via-custodian-sync");

pub struct RecordCollateralUpdateViaCustodianSyncJobInitializer<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, E> RecordCollateralUpdateViaCustodianSyncJobInitializer<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(ledger: Arc<CollateralLedger>, repo: Arc<CollateralRepo<E>>) -> Self {
        Self {
            repo,
            ledger,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, E> JobInitializer for RecordCollateralUpdateViaCustodianSyncJobInitializer<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = RecordCollateralUpdateViaCustodianSyncConfig;

    fn job_type(&self) -> JobType {
        RECORD_COLLATERAL_UPDATE_VIA_CUSTODIAN_SYNC_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RecordCollateralUpdateViaCustodianSyncJobRunner::<
            S,
            E,
        > {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            _phantom: std::marker::PhantomData,
        }))
    }
}

pub struct RecordCollateralUpdateViaCustodianSyncJobRunner<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    config: RecordCollateralUpdateViaCustodianSyncConfig,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
    _phantom: std::marker::PhantomData<S>,
}

#[async_trait]
impl<S, E> JobRunner for RecordCollateralUpdateViaCustodianSyncJobRunner<S, E>
where
    S: SystemSubject + Send + Sync + 'static,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_credit.record_collateral_update_via_custodian_sync_job.run",
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
        let mut collateral = self
            .repo
            .find_by_custody_wallet_id(Some(self.config.custody_wallet_id))
            .await?;

        let mut op = current_job.begin_op().await?;

        if let es_entity::Idempotent::Executed(data) = collateral
            .record_collateral_update_via_custodian_sync(
                self.config.updated_collateral,
                self.config.effective,
            )
        {
            self.repo.update_in_op(&mut op, &mut collateral).await?;

            self.ledger
                .update_collateral_amount_in_op(
                    &mut op,
                    data,
                    &S::system(crate::primitives::COLLATERALIZATION_SYNC),
                )
                .await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
