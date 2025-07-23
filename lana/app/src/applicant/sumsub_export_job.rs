use async_trait::async_trait;
use futures::StreamExt;
use tracing::instrument;

use lana_events::LanaEvent;

use crate::{
    applicant::{SUMSUB_EXPORT_JOB, SumsubExportJobData, transaction_export::TransactionExporter},
    customer::CustomerId,
    deposit::{CoreDepositEvent, DepositId, Deposits, WithdrawalId},
    job::*,
    outbox::Outbox,
    primitives::Subject,
};

pub struct SumsubExportInit {
    outbox: Outbox,
    transaction_exporter: TransactionExporter,
    deposits: Deposits,
}

impl SumsubExportInit {
    pub fn new(
        outbox: &Outbox,
        transaction_exporter: TransactionExporter,
        deposits: &Deposits,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            transaction_exporter,
            deposits: deposits.clone(),
        }
    }
}

impl JobInitializer for SumsubExportInit {
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
}

pub struct SumsubExportJobRunner {
    outbox: Outbox,
    transaction_exporter: TransactionExporter,
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
                        .find_account_by_id(&Subject::System, deposit_account_id)
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
                        .find_account_by_id(&Subject::System, deposit_account_id)
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
        amount: core_money::UsdCents,
    ) -> Result<(), crate::applicant::error::ApplicantError> {
        message.inject_trace_parent();
        self.transaction_exporter
            .submit_withdrawal_transaction(withdrawal_id.to_string(), customer_id, amount)
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
        amount: core_money::UsdCents,
    ) -> Result<(), crate::applicant::error::ApplicantError> {
        message.inject_trace_parent();
        self.transaction_exporter
            .submit_deposit_transaction(deposit_id.to_string(), customer_id, amount)
            .await
    }
}
