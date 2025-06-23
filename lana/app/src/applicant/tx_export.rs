use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::SystemSubject;
use core_money::UsdCents;
use lana_events::LanaEvent;

use super::{error::ApplicantError, sumsub_auth::*};
use crate::{
    customer::CustomerId,
    deposit::{CoreDepositEvent, DepositId, Deposits, WithdrawalId},
    job::*,
    outbox::Outbox,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SumsubExportJobConfig;

impl JobConfig for SumsubExportJobConfig {
    type Initializer = SumsubExportJobInitializer;
}

pub struct SumsubExportJobInitializer {
    outbox: Outbox,
    sumsub_client: SumsubClient,
    deposits: Deposits,
}

impl SumsubExportJobInitializer {
    pub fn new(outbox: &Outbox, sumsub_client: &SumsubClient, deposits: &Deposits) -> Self {
        Self {
            outbox: outbox.clone(),
            sumsub_client: sumsub_client.clone(),
            deposits: deposits.clone(),
        }
    }
}

const SUMSUB_EXPORT_JOB: JobType = JobType::new("sumsub-export");
impl JobInitializer for SumsubExportJobInitializer {
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
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct SumsubExportJobData {
    sequence: outbox::EventSequence,
}

pub struct SumsubExportJobRunner {
    outbox: Outbox,
    sumsub_client: SumsubClient,
    deposits: Deposits,
}

#[async_trait]
impl JobRunner for SumsubExportJobRunner {
    #[tracing::instrument(name = "applicant.sumsub_export", skip_all, fields(insert_id), err)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<SumsubExportJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.payload {
                Some(LanaEvent::Deposit(CoreDepositEvent::DepositInitialized {
                    id,
                    deposit_account_id,
                    amount,
                })) => {
                    let account = self
                        .deposits
                        .find_account_by_id(&rbac_types::Subject::system(), deposit_account_id)
                        .await?
                        .expect("Deposit account not found");
                    self.submit_deposit_transaction(
                        &message,
                        id,
                        account.account_holder_id.into(),
                        amount,
                    )
                    .await?
                }
                Some(LanaEvent::Deposit(CoreDepositEvent::WithdrawalConfirmed {
                    id,
                    deposit_account_id,
                    amount,
                })) => {
                    let account = self
                        .deposits
                        .find_account_by_id(&rbac_types::Subject::system(), deposit_account_id)
                        .await?
                        .expect("Deposit account not found");
                    self.submit_withdrawal_transaction(
                        &message,
                        id,
                        account.account_holder_id.into(),
                        amount,
                    )
                    .await?
                }
                _ => continue,
            }
            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }
        Ok(JobCompletion::RescheduleNow)
    }
}

impl SumsubExportJobRunner {
    #[instrument(
        name = "applicants.sumsub_export.submit_withdrawal_transaction",
        skip(self),
        err
    )]
    pub async fn submit_withdrawal_transaction(
        &self,
        message: &outbox::PersistentOutboxEvent<LanaEvent>,
        withdrawal_id: WithdrawalId,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), ApplicantError> {
        message.inject_trace_parent();
        self.sumsub_client
            .submit_finance_transaction(
                customer_id,
                withdrawal_id.to_string(),
                "Withdrawal",
                &SumsubTransactionDirection::Out.to_string(),
                usd_cents_to_dollars(amount),
                "USD",
            )
            .await
    }

    #[instrument(
        name = "applicants.sumsub_export.submit_deposit_transaction",
        skip(self),
        err
    )]
    pub async fn submit_deposit_transaction(
        &self,
        message: &outbox::PersistentOutboxEvent<LanaEvent>,
        deposit_id: DepositId,
        customer_id: CustomerId,
        amount: UsdCents,
    ) -> Result<(), ApplicantError> {
        message.inject_trace_parent();
        self.sumsub_client
            .submit_finance_transaction(
                customer_id,
                deposit_id.to_string(),
                "Deposit",
                &SumsubTransactionDirection::In.to_string(),
                usd_cents_to_dollars(amount),
                "USD",
            )
            .await
    }
}

pub fn usd_cents_to_dollars(cents: UsdCents) -> f64 {
    // Use the into_inner method to get the value in cents
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
