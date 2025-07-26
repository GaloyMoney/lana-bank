use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountHolderId,
    GovernanceAction, GovernanceObject,
};
use core_money::UsdCents;
use governance::GovernanceEvent;
use outbox::{Outbox, OutboxEventMarker};
use sumsub::SumsubClient;

use job::*;
use lana_events::LanaEvent;

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
    sumsub_client: SumsubClient,
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
        sumsub_client: SumsubClient,
        deposits: &CoreDeposit<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            sumsub_client,
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
            sumsub_client: self.sumsub_client.clone(),
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
    sumsub_client: SumsubClient,
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
                    message.inject_trace_parent();
                    let amount_usd: f64 = amount.to_usd().try_into()?;
                    self.sumsub_client
                        .submit_finance_transaction(
                            account.account_holder_id,
                            id.to_string(),
                            "Deposit",
                            &SumsubTransactionDirection::In.to_string(),
                            amount_usd,
                            "USD",
                        )
                        .await?;
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
                    message.inject_trace_parent();
                    let amount_usd: f64 = amount.to_usd().try_into()?;
                    self.sumsub_client
                        .submit_finance_transaction(
                            account.account_holder_id,
                            id.to_string(),
                            "Withdrawal",
                            &SumsubTransactionDirection::Out.to_string(),
                            amount_usd,
                            "USD",
                        )
                        .await?;
                }
                _ => continue,
            }
            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }
        Ok(JobCompletion::RescheduleNow)
    }
}
