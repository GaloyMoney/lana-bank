use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sumsub::SumsubError;

use job::*;

use core_customer::PartyId;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IngestSumsubApplicantConfig {
    pub party_id: PartyId,
}

pub const INGEST_SUMSUB_APPLICANT_COMMAND: JobType =
    JobType::new("command.customer-sync.ingest-sumsub-applicant");

pub struct IngestSumsubApplicantJobInitializer {
    pool: PgPool,
    sumsub_client: sumsub::SumsubClient,
}

impl IngestSumsubApplicantJobInitializer {
    pub fn new(pool: PgPool, sumsub_client: sumsub::SumsubClient) -> Self {
        Self {
            pool,
            sumsub_client,
        }
    }
}

impl JobInitializer for IngestSumsubApplicantJobInitializer {
    type Config = IngestSumsubApplicantConfig;

    fn job_type(&self) -> JobType {
        INGEST_SUMSUB_APPLICANT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(IngestSumsubApplicantJobRunner {
            config: job.config()?,
            pool: self.pool.clone(),
            sumsub_client: self.sumsub_client.clone(),
        }))
    }
}

pub struct IngestSumsubApplicantJobRunner {
    config: IngestSumsubApplicantConfig,
    pool: PgPool,
    sumsub_client: sumsub::SumsubClient,
}

#[async_trait]
impl JobRunner for IngestSumsubApplicantJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.ingest_sumsub_applicant_job.process_command",
        skip(self, _current_job),
        fields(party_id = %self.config.party_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let external_user_id = self.config.party_id.to_string();
        let applicant = match self
            .sumsub_client
            .get_applicant_details(external_user_id.clone())
            .await
        {
            Ok(applicant) => applicant,
            Err(SumsubError::ApiError { code: 404, description }) => {
                tracing::info!(
                    party_id = %self.config.party_id,
                    %external_user_id,
                    %description,
                    "No Sumsub applicant found for party, skipping ingestion"
                );
                return Ok(JobCompletion::Complete);
            }
            Err(err) => return Err(err.into()),
        };

        let applicant_id = applicant.id.clone();
        let applicant_raw = serde_json::to_value(&applicant)?;
        let document_resources = self
            .sumsub_client
            .get_applicant_document_resources(&applicant_id)
            .await?;

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO data_sumsub_applicants (
                applicant_id,
                external_user_id,
                raw_json,
                fetched_at
            )
            VALUES ($1, $2, $3, NOW())
            "#,
        )
        .bind(&applicant_id)
        .bind(&external_user_id)
        .bind(applicant_raw)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO data_sumsub_documents (
                applicant_id,
                external_user_id,
                raw_json,
                fetched_at
            )
            VALUES ($1, $2, $3, NOW())
            "#,
        )
        .bind(&applicant_id)
        .bind(&external_user_id)
        .bind(document_resources)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(JobCompletion::Complete)
    }
}

