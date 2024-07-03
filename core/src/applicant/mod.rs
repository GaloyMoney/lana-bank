mod config;
pub mod error;
mod sumsub_auth;
pub mod sumsub_public;

pub use config::*;

use error::ApplicantError;
use serde::{Deserialize, Serialize};
use sumsub_auth::*;

use crate::{primitives::UserId, user::Users};

#[derive(Clone)]
pub struct Applicants {
    _pool: sqlx::PgPool,
    sumsub_client: SumsubClient,
    users: Users,
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
        external_user_id: UserId,
        review_status: String,
        created_at_ms: String,
        client_id: Option<String>,
    },
    #[serde(rename = "applicantPending")]
    #[serde(rename_all = "camelCase")]
    ApplicantPending {
        applicant_id: String,
        inspection_id: String,
        applicant_type: Option<String>,
        correlation_id: String,
        level_name: SumsubKycLevel,
        external_user_id: UserId,
        review_status: String,
        created_at_ms: String,
        client_id: Option<String>,
    },
    #[serde(rename = "applicantReviewed")]
    #[serde(rename_all = "camelCase")]
    ApplicantReviewed {
        applicant_id: String,
        inspection_id: String,
        correlation_id: String,
        external_user_id: UserId,
        level_name: SumsubKycLevel,
        review_result: ReviewResult,
        review_status: String,
        created_at_ms: String,
    },
    #[serde(rename = "applicantOnHold")]
    #[serde(rename_all = "camelCase")]
    ApplicantOnHold {
        applicant_id: String,
        inspection_id: String,
        applicant_type: Option<String>,
        correlation_id: String,
        level_name: SumsubKycLevel,
        external_user_id: UserId,
        review_result: ReviewResult,
        review_status: String,
        created_at_ms: String,
        client_id: Option<String>,
    },
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
    pub fn new(pool: &sqlx::PgPool, config: &SumsubConfig, users: &Users) -> Self {
        let sumsub_client = SumsubClient::new(&config);
        Self {
            _pool: pool.clone(),
            sumsub_client,
            users: users.clone(),
        }
    }

    pub async fn handle_callback(&self, payload: serde_json::Value) -> Result<(), ApplicantError> {
        println!("handle sumsub callback");

        let payload = match serde_json::from_value::<SumsubCallbackPayload>(payload) {
            Ok(payload) => payload,
            Err(err) => {
                return Err(ApplicantError::InvalidPayload(err));
            }
        };

        match payload {
            SumsubCallbackPayload::ApplicantCreated {
                external_user_id,
                applicant_id,
                ..
            } => {
                println!(
                    "No KYC status update for user_id {}, applicant_id: {}",
                    external_user_id, applicant_id
                );

                Ok(())
            }
            SumsubCallbackPayload::ApplicantReviewed {
                external_user_id,
                review_result,
                applicant_id,
                level_name,
                ..
            } => {
                match review_result.review_answer {
                    ReviewAnswer::Red => {
                        let res = self.users.deactivate(external_user_id, applicant_id).await;

                        if let Err(err) = res {
                            return Err(ApplicantError::UpdatingEntryError(err.to_string()));
                        }
                    }
                    ReviewAnswer::Green => match level_name {
                        SumsubKycLevel::BasicKycLevel => {
                            let res = self
                                .users
                                .approve_basic(external_user_id, applicant_id)
                                .await;

                            if let Err(err) = res {
                                return Err(ApplicantError::UpdatingEntryError(err.to_string()));
                            }
                        }
                        SumsubKycLevel::AdvancedKycLevel => {
                            todo!("implement advanced kyc level")
                        }
                    },
                };

                Ok(())
            }
            SumsubCallbackPayload::ApplicantOnHold {
                external_user_id,
                review_result,
                applicant_id,
                level_name,
                ..
            } => {
                println!("applicant on hold: {}", external_user_id);
                println!("review result: {:?}", review_result);
                println!("applicant_id: {}", applicant_id);
                println!("level_name: {}", level_name);
                todo!()
            }
            // no op
            SumsubCallbackPayload::ApplicantPending { .. } => Ok(()),
        }
    }

    pub async fn create_access_token(
        &self,
        user_id: UserId,
    ) -> Result<CreateAccessTokenResponse, anyhow::Error> {
        let client = reqwest::Client::new();

        let level_name = SumsubKycLevel::BasicKycLevel;

        self.sumsub_client
            .create_access_token(&client, user_id, &level_name.to_string())
            .await
    }

    pub async fn create_permalink(
        &self,
        user_id: UserId,
    ) -> Result<CreatePermalinkResponse, anyhow::Error> {
        let client = reqwest::Client::new();

        let level_name = SumsubKycLevel::BasicKycLevel;

        self.sumsub_client
            .create_permalink(&client, user_id, &level_name.to_string())
            .await
    }
}
