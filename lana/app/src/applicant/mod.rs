mod config;
pub mod error;
mod job;
mod repo;
mod sumsub_auth;

use job::{SumsubExportConfig, SumsubExportInitializer};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    customer::Customers,
    job::Jobs,
    primitives::{CustomerId, JobId},
};

pub use config::*;
use error::ApplicantError;
use sumsub_auth::*;

use repo::ApplicantRepo;
pub use sumsub_auth::{AccessTokenResponse, PermalinkResponse};

#[derive(Clone)]
pub struct Applicants {
    sumsub_client: SumsubClient,
    users: Customers,
    repo: ApplicantRepo,
    jobs: Jobs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReviewAnswer {
    Green,
    Red,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SumsubKycLevel {
    BasicKycLevel,
    AdvancedKycLevel,
}

impl std::fmt::Display for SumsubKycLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SumsubKycLevel::BasicKycLevel => write!(f, "basic-kyc-level"),
            SumsubKycLevel::AdvancedKycLevel => write!(f, "advanced-kyc-level"),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(tag = "type")]
pub enum SumsubCallbackPayload {
    #[serde(rename = "applicantCreated")]
    #[serde(rename_all = "camelCase")]
    ApplicantCreated {
        applicant_id: String,
        inspection_id: String,
        correlation_id: String,
        level_name: SumsubKycLevel,
        external_user_id: CustomerId,
        review_status: String,
        created_at_ms: String,
        client_id: Option<String>,
        sandbox_mode: Option<bool>,
    },
    #[serde(rename = "applicantReviewed")]
    #[serde(rename_all = "camelCase")]
    ApplicantReviewed {
        applicant_id: String,
        inspection_id: String,
        correlation_id: String,
        external_user_id: CustomerId,
        level_name: SumsubKycLevel,
        review_result: ReviewResult,
        review_status: String,
        created_at_ms: String,
        sandbox_mode: Option<bool>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub review_answer: ReviewAnswer,
    pub moderation_comment: Option<String>,
    pub client_comment: Option<String>,
    pub reject_labels: Option<Vec<String>>,
    pub review_reject_type: Option<String>,
}

impl Applicants {
    pub fn new(
        pool: &sqlx::PgPool,
        config: &SumsubConfig,
        users: &Customers,
        jobs: &Jobs,
        // export: &Export,
    ) -> Self {
        let sumsub_client = SumsubClient::new(config);
        jobs.add_initializer(SumsubExportInitializer::new(
            // export.clone(),
            sumsub_client.clone(),
            pool,
        ));

        Self {
            repo: ApplicantRepo::new(pool),
            sumsub_client,
            users: users.clone(),
            jobs: jobs.clone(),
        }
    }

    pub async fn handle_callback(&self, payload: serde_json::Value) -> Result<(), ApplicantError> {
        let customer_id: CustomerId = payload["externalUserId"]
            .as_str()
            .ok_or_else(|| ApplicantError::MissingExternalUserId(payload.to_string()))?
            .parse()?;

        let callback_id = &self
            .repo
            .persist_webhook_data(customer_id, payload.clone())
            .await?;

        let mut db = self.repo.begin_op().await?;

        self.jobs
            .create_and_spawn_in_op(
                &mut db,
                JobId::new(),
                SumsubExportConfig::Webhook {
                    callback_id: *callback_id,
                },
            )
            .await?;

        match self.process_payload(&mut db, payload).await {
            Ok(_) => (),
            Err(ApplicantError::UnhandledCallbackType(_)) => (),
            Err(e) => return Err(e),
        }

        db.commit().await?;

        Ok(())
    }

    async fn process_payload(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payload: serde_json::Value,
    ) -> Result<(), ApplicantError> {
        match serde_json::from_value(payload.clone())? {
            SumsubCallbackPayload::ApplicantCreated {
                external_user_id,
                applicant_id,
                sandbox_mode,
                ..
            } => {
                let res = self
                    .users
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
            SumsubCallbackPayload::ApplicantReviewed {
                external_user_id,
                review_result:
                    ReviewResult {
                        review_answer: ReviewAnswer::Red,
                        ..
                    },
                applicant_id,
                sandbox_mode,
                ..
            } => {
                let res = self
                    .users
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
            SumsubCallbackPayload::ApplicantReviewed {
                external_user_id,
                review_result:
                    ReviewResult {
                        review_answer: ReviewAnswer::Green,
                        ..
                    },
                applicant_id,
                level_name: SumsubKycLevel::BasicKycLevel,
                sandbox_mode,
                ..
            } => {
                let res = self
                    .users
                    .approve_kyc(db, external_user_id, applicant_id)
                    .await;

                match res {
                    Ok(_) => (),
                    Err(e) if e.was_not_found() && sandbox_mode.unwrap_or(false) => {
                        return Ok(());
                    }
                    Err(e) => return Err(e.into()),
                }

                self.jobs
                    .create_and_spawn_in_op(
                        db,
                        JobId::new(),
                        SumsubExportConfig::SensitiveInfo {
                            customer_id: external_user_id,
                        },
                    )
                    .await?;
            }
            SumsubCallbackPayload::ApplicantReviewed {
                review_result:
                    ReviewResult {
                        review_answer: ReviewAnswer::Green,
                        ..
                    },
                level_name: SumsubKycLevel::AdvancedKycLevel,
                ..
            } => {
                return Err(ApplicantError::UnhandledCallbackType(
                    "Advanced KYC level is not supported".to_string(),
                ));
            }
            SumsubCallbackPayload::Unknown => {
                return Err(ApplicantError::UnhandledCallbackType(format!(
                    "callback event not processed for payload {payload}",
                )));
            }
        }
        Ok(())
    }

    pub async fn create_access_token(
        &self,
        customer_id: CustomerId,
    ) -> Result<AccessTokenResponse, ApplicantError> {
        let level_name = SumsubKycLevel::BasicKycLevel;

        self.sumsub_client
            .create_access_token(customer_id, &level_name.to_string())
            .await
    }

    #[instrument(name = "applicant.create_permalink", skip(self))]
    pub async fn create_permalink(
        &self,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<PermalinkResponse, ApplicantError> {
        let level_name = SumsubKycLevel::BasicKycLevel;

        self.sumsub_client
            .create_permalink(customer_id.into(), &level_name.to_string())
            .await
    }
}
