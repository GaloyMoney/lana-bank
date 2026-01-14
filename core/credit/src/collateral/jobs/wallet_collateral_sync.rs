use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use std::sync::Arc;

use job::*;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use core_custody::CoreCustodyEvent;

use crate::{
    CoreCreditEvent,
    collateral::{CollateralError, CollateralRepo},
    ledger::CreditLedger,
};

#[derive(Serialize, Deserialize)]
pub struct WalletCollateralSyncJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for WalletCollateralSyncJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> WalletCollateralSyncJobConfig<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> Default for WalletCollateralSyncJobConfig<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct WalletCollateralSyncJobData {
    sequence: obix::EventSequence,
}

pub struct WalletCollateralSyncInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CreditLedger>,
}

impl<E> WalletCollateralSyncInit<E>
where
    E: OutboxEventMarker<CoreCustodyEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        ledger: Arc<CreditLedger>,
        collateral_repo: Arc<CollateralRepo<E>>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            ledger,
            repo: collateral_repo,
        }
    }
}

const WALLET_COLLATERAL_SYNC_JOB: JobType = JobType::new("outbox.wallet-collateral-sync");
impl<E> JobInitializer for WalletCollateralSyncInit<E>
where
    E: OutboxEventMarker<CoreCustodyEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    type Config = WalletCollateralSyncJobConfig<E>;

    fn job_type(&self) -> JobType {
        WALLET_COLLATERAL_SYNC_JOB
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(WalletCollateralSyncJobRunner {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct WalletCollateralSyncJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CreditLedger>,
    outbox: Outbox<E>,
}

impl<E> WalletCollateralSyncJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "core_credit.wallet_collateral_sync_job.process_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[allow(clippy::single_match)]
        match message.as_event() {
            Some(
                event @ CoreCustodyEvent::WalletBalanceChanged {
                    id,
                    new_balance,
                    changed_at,
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                self.record_collateral_update_via_custodian_sync(
                    *id,
                    *new_balance,
                    changed_at.date_naive(),
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.record_collateral_update_via_custodian_sync",
        fields(updated_collateral = %updated_collateral, effective = %effective),
        skip(self),
    )]
    async fn record_collateral_update_via_custodian_sync(
        &self,
        custody_wallet_id: crate::primitives::CustodyWalletId,
        updated_collateral: core_money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<(), CollateralError> {
        let mut collateral = self
            .repo
            .find_by_custody_wallet_id(Some(custody_wallet_id))
            .await?;

        let mut db = self.repo.begin_op().await?;

        if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_custodian_sync(updated_collateral, effective)
        {
            self.repo.update_in_op(&mut db, &mut collateral).await?;

            self.ledger
                .update_credit_facility_collateral(
                    &mut db,
                    data,
                    collateral.account_id,
                    core_accounting::LedgerTransactionInitiator::System,
                )
                .await?;
            db.commit().await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for WalletCollateralSyncJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<WalletCollateralSyncJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %WALLET_COLLATERAL_SYNC_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            self.process_message(message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job.update_execution_state(&state).await?;
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
