use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

use core_applicant;
use lana_events::LanaEvent;

use crate::{
    customer::{CustomerId, Customers},
    deposit::{CoreDepositEvent, Deposits},
    job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, Jobs},
    outbox::Outbox,
    primitives::Subject,
};

pub use core_applicant::{
    ApplicantInfo, PermalinkResponse, ReviewAnswer, SumsubConfig, SumsubVerificationLevel,
    TransactionData, TransactionExporter, TransactionProcessor, TransactionType, error,
};

#[cfg(feature = "sumsub-testing")]
pub use core_applicant::{SumsubClient as PublicSumsubClient, sumsub_testing_utils};

/// Application-level Applicants service that wraps the core functionality
/// and provides integration with other services
#[derive(Clone)]
pub struct Applicants {
    core_applicants: core_applicant::Applicants,
    customers: Arc<Customers>,
}

impl Applicants {
    pub async fn init(
        pool: &sqlx::PgPool,
        config: &SumsubConfig,
        customers: &Customers,
        deposits: &Deposits,
        jobs: &Jobs,
        outbox: &Outbox,
    ) -> Result<Self, error::ApplicantError> {
        let core_applicants = core_applicant::Applicants::new(pool, config);

        // Initialize the application-level job system for transaction export
        jobs.add_initializer_and_spawn_unique(
            SumsubExportInit::new(outbox, core_applicants.transaction_exporter(), deposits),
            SumsubExportJobConfig,
        )
        .await?;

        Ok(Self {
            core_applicants,
            customers: Arc::new(customers.clone()),
        })
    }

    #[instrument(name = "applicant.handle_callback", skip(self, payload))]
    pub async fn handle_callback(
        &self,
        payload: serde_json::Value,
    ) -> Result<(), error::ApplicantError> {
        // First, persist the webhook data using the core service
        self.core_applicants
            .handle_callback(payload.clone())
            .await?;

        // Now process the payload for customer operations
        let _customer_id: CustomerId = payload["externalUserId"]
            .as_str()
            .ok_or_else(|| error::ApplicantError::MissingExternalUserId(payload.to_string()))?
            .parse()?;

        // Process the payload for customer operations
        let mut db = self.customers.repo().begin_op().await?;

        match self.process_payload(&mut db, payload).await {
            Ok(_) => (),
            Err(error::ApplicantError::UnhandledCallbackType(_)) => (),
            Err(e) => return Err(e),
        }

        db.commit().await?;

        Ok(())
    }

    pub async fn process_payload(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payload: serde_json::Value,
    ) -> Result<(), error::ApplicantError> {
        match serde_json::from_value(payload.clone())? {
            core_applicant::SumsubCallbackPayload::ApplicantCreated {
                external_user_id,
                applicant_id,
                sandbox_mode,
                ..
            } => {
                let res = self
                    .customers
                    .start_kyc(db, external_user_id, applicant_id)
                    .await;

                match res {
                    Ok(_) => (),
                    Err(e) if e.was_not_found() && sandbox_mode.unwrap_or(false) => {
                        return Ok(());
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            core_applicant::SumsubCallbackPayload::ApplicantReviewed {
                external_user_id,
                review_result:
                    core_applicant::ReviewResult {
                        review_answer: core_applicant::ReviewAnswer::Red,
                        ..
                    },
                applicant_id,
                sandbox_mode,
                ..
            } => {
                let res = self
                    .customers
                    .decline_kyc(db, external_user_id, applicant_id)
                    .await;

                match res {
                    Ok(_) => (),
                    Err(e) if e.was_not_found() && sandbox_mode.unwrap_or(false) => {
                        return Ok(());
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            core_applicant::SumsubCallbackPayload::ApplicantReviewed {
                external_user_id,
                review_result:
                    core_applicant::ReviewResult {
                        review_answer: core_applicant::ReviewAnswer::Green,
                        ..
                    },
                applicant_id,
                level_name,
                sandbox_mode,
                ..
            } => {
                // Try to parse the level name, will return error for unrecognized values
                match level_name.parse::<SumsubVerificationLevel>() {
                    Ok(_) => {} // Level is valid, continue
                    Err(_) => {
                        return Err(error::ApplicantError::UnhandledCallbackType(format!(
                            "Sumsub level {level_name} not implemented"
                        )));
                    }
                };

                let res = self
                    .customers
                    .approve_kyc(db, external_user_id, applicant_id)
                    .await;

                match res {
                    Ok(_) => (),
                    Err(e) if e.was_not_found() && sandbox_mode.unwrap_or(false) => {
                        return Ok(());
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            core_applicant::SumsubCallbackPayload::Unknown => {
                return Err(error::ApplicantError::UnhandledCallbackType(format!(
                    "callback event not processed for payload {payload}",
                )));
            }
        }
        Ok(())
    }

    #[instrument(name = "applicant.create_permalink", skip(self))]
    pub async fn create_permalink(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<PermalinkResponse, error::ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        let customer = self.customers.find_by_id(sub, customer_id).await?;
        let customer = customer.ok_or_else(|| {
            error::ApplicantError::CustomerIdNotFound(format!(
                "Customer with ID {customer_id} not found"
            ))
        })?;

        let level: SumsubVerificationLevel = customer.customer_type.into();

        self.core_applicants
            .create_permalink(customer_id, &level.to_string())
            .await
    }

    #[instrument(name = "applicant.get_applicant_info", skip(self))]
    pub async fn get_applicant_info(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<ApplicantInfo, error::ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        // TODO: audit
        self.customers.find_by_id_without_audit(customer_id).await?;

        self.core_applicants.get_applicant_info(customer_id).await
    }

    /// Creates a complete test applicant with documents and approval for testing purposes
    /// This method executes the full KYC flow automatically using predefined test data
    #[cfg(feature = "sumsub-testing")]
    #[instrument(name = "applicant.create_complete_test_applicant", skip(self))]
    pub async fn create_complete_test_applicant(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<String, error::ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        let customer = self.customers.find_by_id_without_audit(customer_id).await?;
        let level: SumsubVerificationLevel = customer.customer_type.into();

        self.core_applicants
            .create_complete_test_applicant(customer_id, &level.to_string())
            .await
    }
}

// Application-level job system for Sumsub transaction export
#[derive(Clone, Serialize, Deserialize)]
pub struct SumsubExportJobConfig;

impl JobConfig for SumsubExportJobConfig {
    type Initializer = SumsubExportInit;
}

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

const SUMSUB_EXPORT_JOB: JobType = JobType::new("sumsub-export");

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

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct SumsubExportJobData {
    sequence: outbox::EventSequence,
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
                        .find_account_by_id(&rbac_types::Subject::System, deposit_account_id)
                        .await?
                        .expect("Deposit account not found");
                    message.inject_trace_parent();
                    let transaction_data = TransactionData::new_deposit(
                        id.to_string(),
                        account.account_holder_id.into(),
                        amount,
                    );
                    self.transaction_exporter
                        .process_transaction(transaction_data)
                        .await?
                }
                Some(LanaEvent::Deposit(CoreDepositEvent::WithdrawalConfirmed {
                    id,
                    deposit_account_id,
                    amount,
                })) => {
                    let account = self
                        .deposits
                        .find_account_by_id(&rbac_types::Subject::System, deposit_account_id)
                        .await?
                        .expect("Deposit account not found");
                    message.inject_trace_parent();
                    let transaction_data = TransactionData::new_withdrawal(
                        id.to_string(),
                        account.account_holder_id.into(),
                        amount,
                    );
                    self.transaction_exporter
                        .process_transaction(transaction_data)
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
