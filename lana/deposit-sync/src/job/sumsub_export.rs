use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::CustomerId;
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositId,
    GovernanceAction, GovernanceObject, WithdrawalId,
};
use core_money::UsdCents;
use governance::GovernanceEvent;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};
use sumsub::SumsubClient;

use job::*;
use lana_events::LanaEvent;

use crate::{config::*, error::DepositSyncError};

/// Job configuration for Sumsub export
pub const SUMSUB_EXPORT_JOB: JobType = JobType::new("sumsub-export");

/// Converts USD cents to dollars for Sumsub API
pub fn usd_cents_to_dollars(cents: UsdCents) -> f64 {
    (cents.into_inner() as f64) / 100.0
}

/// Direction of the transaction from Sumsub's perspective
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SumsubTransactionDirection {
    /// Money coming into the customer's account (deposit)
    #[serde(rename = "in")]
    In,
    /// Money going out of the customer's account (withdrawal)
    #[serde(rename = "out")]
    Out,
}

impl std::fmt::Display for SumsubTransactionDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SumsubTransactionDirection::In => write!(f, "in"),
            SumsubTransactionDirection::Out => write!(f, "out"),
        }
    }
}

/// Transaction data for export to Sumsub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SumsubExportJobData {
    pub transaction_id: String,
    pub customer_id: CustomerId,
    pub amount: UsdCents,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
}

/// Sumsub transaction exporter
#[derive(Clone)]
pub struct SumsubTransactionExporter {
    sumsub_client: SumsubClient,
}

impl SumsubTransactionExporter {
    pub fn new(sumsub_client: SumsubClient) -> Self {
        Self { sumsub_client }
    }

    /// Submit a deposit transaction to Sumsub for monitoring
    #[instrument(
        name = "deposit_sync.submit_deposit_transaction_to_sumsub",
        skip(self),
        err
    )]
    pub async fn submit_deposit_transaction(
        &self,
        transaction_id: impl Into<String> + std::fmt::Debug,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        let transaction_id = transaction_id.into();

        self.sumsub_client
            .submit_finance_transaction(
                customer_id,
                transaction_id,
                "Deposit",
                &SumsubTransactionDirection::In.to_string(),
                usd_cents_to_dollars(amount),
                "USD",
            )
            .await
            .map_err(DepositSyncError::from)
    }

    /// Submit a withdrawal transaction to Sumsub for monitoring
    #[instrument(
        name = "deposit_sync.submit_withdrawal_transaction_to_sumsub",
        skip(self),
        err
    )]
    pub async fn submit_withdrawal_transaction(
        &self,
        transaction_id: impl Into<String> + std::fmt::Debug,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        let transaction_id = transaction_id.into();

        self.sumsub_client
            .submit_finance_transaction(
                customer_id,
                transaction_id,
                "Withdrawal",
                &SumsubTransactionDirection::Out.to_string(),
                usd_cents_to_dollars(amount),
                "USD",
            )
            .await
            .map_err(DepositSyncError::from)
    }
}

#[derive(serde::Serialize)]
pub struct SumsubExportJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> SumsubExportJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> JobConfig for SumsubExportJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    type Initializer = SumsubExportInit<Perms, E>;
}

pub struct SumsubExportInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    outbox: Outbox<E>,
    transaction_exporter: SumsubTransactionExporter,
    deposits: CoreDeposit<Perms, E>,
    config: DepositSyncConfig,
}

impl<Perms, E> SumsubExportInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    pub fn new(
        outbox: &Outbox<E>,
        transaction_exporter: SumsubTransactionExporter,
        deposits: &CoreDeposit<Perms, E>,
        config: DepositSyncConfig,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            transaction_exporter,
            deposits: deposits.clone(),
            config,
        }
    }
}

impl<Perms, E> JobInitializer for SumsubExportInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        SUMSUB_EXPORT_JOB
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SumsubExportJobRunner {
            outbox: self.outbox.clone(),
            transaction_exporter: self.transaction_exporter.clone(),
            deposits: self.deposits.clone(),
            config: self.config.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct SumsubExportJobState {
    sequence: outbox::EventSequence,
}

pub struct SumsubExportJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    outbox: Outbox<E>,
    transaction_exporter: SumsubTransactionExporter,
    deposits: CoreDeposit<Perms, E>,
    config: DepositSyncConfig,
}

#[async_trait]
impl<Perms, E> JobRunner for SumsubExportJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    #[tracing::instrument(name = "deposit_sync.sumsub_export", skip_all, fields(insert_id), err)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if !self.config.sumsub_export_enabled {
            return Ok(JobCompletion::RescheduleNow);
        }

        let mut state = current_job
            .execution_state::<SumsubExportJobState>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.as_ref().as_event() {
                Some(LanaEvent::Deposit(CoreDepositEvent::DepositInitialized {
                    id,
                    deposit_account_id,
                    amount,
                })) => {
                    let account = self
                        .deposits
                        .find_account_by_id(
                            &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(),
                            *deposit_account_id,
                        )
                        .await?
                        .expect("Deposit account not found");
                    self.submit_deposit_transaction(
                        &message,
                        *id,
                        account.account_holder_id.into(),
                        *amount,
                    )
                    .await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(&state).await?;
                }
                Some(LanaEvent::Deposit(CoreDepositEvent::WithdrawalConfirmed {
                    id,
                    deposit_account_id,
                    amount,
                })) => {
                    let account = self
                        .deposits
                        .find_account_by_id(
                            &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(),
                            *deposit_account_id,
                        )
                        .await?
                        .expect("Deposit account not found");
                    self.submit_withdrawal_transaction(
                        &message,
                        *id,
                        account.account_holder_id.into(),
                        *amount,
                    )
                    .await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(&state).await?;
                }
                _ => continue,
            }
        }
        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> SumsubExportJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    #[instrument(name = "deposit_sync.submit_withdrawal_transaction", skip(self), err)]
    pub async fn submit_withdrawal_transaction(
        &self,
        message: &PersistentOutboxEvent<E>,
        withdrawal_id: WithdrawalId,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        message.inject_trace_parent();
        self.transaction_exporter
            .submit_withdrawal_transaction(withdrawal_id.to_string(), customer_id, amount)
            .await
    }

    #[instrument(name = "deposit_sync.submit_deposit_transaction", skip(self), err)]
    pub async fn submit_deposit_transaction(
        &self,
        message: &PersistentOutboxEvent<E>,
        deposit_id: DepositId,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        message.inject_trace_parent();
        self.transaction_exporter
            .submit_deposit_transaction(deposit_id.to_string(), customer_id, amount)
            .await
    }
}
