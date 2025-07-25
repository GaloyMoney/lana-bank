use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountHolderId,
    DepositId, GovernanceAction, GovernanceObject, WithdrawalId,
};
use core_money::UsdCents;
use governance::GovernanceEvent;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};
use sumsub::SumsubClient;

use job::*;
use lana_events::LanaEvent;

use crate::error::DepositSyncError;

/// Job configuration for Sumsub export
pub const SUMSUB_EXPORT_JOB: JobType = JobType::new("sumsub-export");

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
    pub deposit_account_holder_id: DepositAccountHolderId,
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
    #[instrument(name = "deposit_sync.submit_deposit_transaction", skip(self), err)]
    pub async fn submit_deposit_transaction(
        &self,
        transaction_id: impl Into<String> + std::fmt::Debug,
        deposit_account_holder_id: DepositAccountHolderId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        let transaction_id = transaction_id.into();

        self.sumsub_client
            .submit_finance_transaction(
                deposit_account_holder_id,
                transaction_id,
                "Deposit",
                &SumsubTransactionDirection::In.to_string(),
                amount
                    .to_usd()
                    .try_into()
                    .expect("USD amount should convert to f64"),
                "USD",
            )
            .await
            .map_err(DepositSyncError::from)
    }

    /// Submit a withdrawal transaction to Sumsub for monitoring
    #[instrument(name = "deposit_sync.submit_withdrawal_transaction", skip(self), err)]
    pub async fn submit_withdrawal_transaction(
        &self,
        transaction_id: impl Into<String> + std::fmt::Debug,
        deposit_account_holder_id: DepositAccountHolderId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        let transaction_id = transaction_id.into();

        self.sumsub_client
            .submit_finance_transaction(
                deposit_account_holder_id,
                transaction_id,
                "Withdrawal",
                &SumsubTransactionDirection::Out.to_string(),
                amount
                    .to_usd()
                    .try_into()
                    .expect("USD amount should convert to f64"),
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
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            transaction_exporter,
            deposits: deposits.clone(),
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
                        .find_account_by_id_without_audit(*deposit_account_id)
                        .await?
                        .expect("Deposit account not found");
                    self.submit_deposit_transaction(
                        &message,
                        *id,
                        account.account_holder_id,
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
                        .find_account_by_id_without_audit(*deposit_account_id)
                        .await?
                        .expect("Deposit account not found");
                    self.submit_withdrawal_transaction(
                        &message,
                        *id,
                        account.account_holder_id,
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
        deposit_account_holder_id: DepositAccountHolderId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        message.inject_trace_parent();
        self.transaction_exporter
            .submit_withdrawal_transaction(
                withdrawal_id.to_string(),
                deposit_account_holder_id,
                amount,
            )
            .await
    }

    #[instrument(name = "deposit_sync.submit_deposit_transaction", skip(self), err)]
    pub async fn submit_deposit_transaction(
        &self,
        message: &PersistentOutboxEvent<E>,
        deposit_id: DepositId,
        deposit_account_holder_id: DepositAccountHolderId,
        amount: UsdCents,
    ) -> Result<(), DepositSyncError> {
        message.inject_trace_parent();
        self.transaction_exporter
            .submit_deposit_transaction(deposit_id.to_string(), deposit_account_holder_id, amount)
            .await
    }
}
