#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod error;
pub mod job_types;
mod repo;
mod sumsub_auth;
pub mod transaction_export;

#[cfg(feature = "sumsub-testing")]
pub mod sumsub_testing_utils;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;

use core_customer::CustomerId;

pub use config::*;
use error::ApplicantError;

pub use job_types::{SumsubExportJobConfig, SumsubExportJobData, SUMSUB_EXPORT_JOB};
use repo::ApplicantRepo;
use sumsub_auth::SumsubClient;
pub use sumsub_auth::{ApplicantInfo, PermalinkResponse};

use rbac_types::Subject;

#[cfg(feature = "graphql")]
use async_graphql::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum ReviewAnswer {
    Green,
    Red,
}

impl std::str::FromStr for ReviewAnswer {
    type Err = ApplicantError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GREEN" => Ok(ReviewAnswer::Green),
            "RED" => Ok(ReviewAnswer::Red),
            _ => Err(ApplicantError::ReviewAnswerParseError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, strum::Display)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SumsubVerificationLevel {
    #[serde(rename = "basic-kyc-level")]
    #[strum(serialize = "basic-kyc-level")]
    BasicKycLevel,
    #[serde(rename = "basic-kyb-level")]
    #[strum(serialize = "basic-kyb-level")]
    BasicKybLevel,
    Unimplemented,
}

impl std::str::FromStr for SumsubVerificationLevel {
    type Err = ApplicantError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "basic-kyc-level" => Ok(SumsubVerificationLevel::BasicKycLevel),
            "basic-kyb-level" => Ok(SumsubVerificationLevel::BasicKybLevel),
            _ => {
                tracing::warn!("Unrecognized SumsubVerificationLevel: {}", s);
                Err(ApplicantError::SumsubVerificationLevelParseError(
                    s.to_string(),
                ))
            }
        }
    }
}

impl From<&core_customer::CustomerType> for SumsubVerificationLevel {
    fn from(customer_type: &core_customer::CustomerType) -> Self {
        match customer_type {
            core_customer::CustomerType::Individual => SumsubVerificationLevel::BasicKycLevel,
            // Every company types is tied to the same SumSub verification level
            core_customer::CustomerType::GovernmentEntity => SumsubVerificationLevel::BasicKybLevel,
            core_customer::CustomerType::PrivateCompany => SumsubVerificationLevel::BasicKybLevel,
            core_customer::CustomerType::Bank => SumsubVerificationLevel::BasicKybLevel,
            core_customer::CustomerType::FinancialInstitution => {
                SumsubVerificationLevel::BasicKybLevel
            }
            core_customer::CustomerType::ForeignAgencyOrSubsidiary => {
                SumsubVerificationLevel::BasicKybLevel
            }
            core_customer::CustomerType::NonDomiciledCompany => {
                SumsubVerificationLevel::BasicKybLevel
            }
        }
    }
}

impl From<core_customer::CustomerType> for SumsubVerificationLevel {
    fn from(customer_type: core_customer::CustomerType) -> Self {
        (&customer_type).into()
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
        level_name: String,
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
        level_name: String,
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

/// Applicants service
#[derive(Clone)]
pub struct Applicants {
    sumsub_client: SumsubClient,
    repo: ApplicantRepo,
}

impl Applicants {
    pub fn new(pool: &PgPool, config: &SumsubConfig) -> Self {
        let sumsub_client = SumsubClient::new(config);

        Self {
            repo: ApplicantRepo::new(pool),
            sumsub_client,
        }
    }

    #[instrument(name = "applicant.handle_callback", skip(self, payload))]
    pub async fn handle_callback(&self, payload: serde_json::Value) -> Result<(), ApplicantError> {
        let customer_id: CustomerId = payload["externalUserId"]
            .as_str()
            .ok_or_else(|| ApplicantError::MissingExternalUserId(payload.to_string()))?
            .parse()?;

        self.repo
            .persist_webhook_data(customer_id, payload.clone())
            .await?;

        // Note: The callback processing logic will need to be handled at the application layer
        // since it requires access to customer services that are not available in this core crate

        Ok(())
    }

    #[instrument(name = "applicant.create_permalink", skip(self))]
    pub async fn create_permalink(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        level_name: &str,
    ) -> Result<PermalinkResponse, ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        self.sumsub_client
            .create_permalink(customer_id, level_name)
            .await
    }

    #[instrument(name = "applicant.get_applicant_info", skip(self))]
    pub async fn get_applicant_info(
        &self,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<ApplicantInfo, ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        let applicant_details = self
            .sumsub_client
            .get_applicant_details(customer_id)
            .await?;

        Ok(applicant_details.info)
    }

    /// Creates a complete test applicant with documents and approval for testing purposes
    /// This method executes the full KYC flow automatically using predefined test data
    #[cfg(feature = "sumsub-testing")]
    #[instrument(name = "applicant.create_complete_test_applicant", skip(self))]
    pub async fn create_complete_test_applicant(
        &self,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        level_name: &str,
    ) -> Result<String, ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        tracing::info!(
            customer_id = %customer_id,
            "Creating complete test applicant with full KYC flow"
        );

        // Step 1: Create applicant via API
        let applicant_id = self
            .sumsub_client
            .create_applicant(customer_id, level_name)
            .await?;

        tracing::info!(applicant_id = %applicant_id, "Applicant created");

        // Step 2: Update applicant personal information
        self.sumsub_client
            .update_applicant_info(
                &applicant_id,
                sumsub_testing_utils::TEST_FIRST_NAME,
                sumsub_testing_utils::TEST_LAST_NAME,
                sumsub_testing_utils::TEST_DATE_OF_BIRTH,
                sumsub_testing_utils::TEST_COUNTRY_CODE,
            )
            .await?;

        tracing::info!("Applicant personal info updated");

        // Step 3: Upload passport documents (front and back)
        let passport_image = sumsub_testing_utils::load_test_document(
            sumsub_testing_utils::PASSPORT_FILENAME,
            sumsub_testing_utils::GERMAN_PASSPORT_URL,
            "German passport image",
        )
        .await
        .map_err(|e| ApplicantError::Sumsub {
            description: format!("Failed to load passport image: {e}"),
            code: 500,
        })?;

        self.sumsub_client
            .upload_document(
                &applicant_id,
                sumsub_auth::DOC_TYPE_PASSPORT,
                sumsub_auth::DOC_SUBTYPE_FRONT_SIDE,
                Some(sumsub_testing_utils::TEST_COUNTRY_CODE),
                passport_image.clone(),
                sumsub_testing_utils::PASSPORT_FILENAME,
            )
            .await?;

        // Brief delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        self.sumsub_client
            .upload_document(
                &applicant_id,
                sumsub_auth::DOC_TYPE_PASSPORT,
                sumsub_auth::DOC_SUBTYPE_BACK_SIDE,
                Some(sumsub_testing_utils::TEST_COUNTRY_CODE),
                passport_image.clone(),
                sumsub_testing_utils::PASSPORT_FILENAME,
            )
            .await?;

        tracing::info!("Passport documents uploaded");

        // Brief delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Step 4: Upload selfie
        self.sumsub_client
            .upload_document(
                &applicant_id,
                sumsub_auth::DOC_TYPE_SELFIE,
                "",
                Some(sumsub_testing_utils::TEST_COUNTRY_CODE),
                passport_image, // Reuse passport image as selfie for testing
                sumsub_testing_utils::PASSPORT_FILENAME,
            )
            .await?;

        tracing::info!("Selfie uploaded");

        // Brief delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Step 5: Upload proof of residence
        let poa_image = sumsub_testing_utils::load_test_document(
            sumsub_testing_utils::POA_FILENAME,
            sumsub_testing_utils::POA_DOCUMENT_URL,
            "Proof of residence document",
        )
        .await
        .map_err(|e| ApplicantError::Sumsub {
            description: format!("Failed to load proof of residence image: {e}"),
            code: 500,
        })?;

        self.sumsub_client
            .upload_document(
                &applicant_id,
                sumsub_auth::DOC_TYPE_UTILITY_BILL,
                "",
                Some(sumsub_testing_utils::TEST_COUNTRY_CODE),
                poa_image,
                sumsub_testing_utils::POA_FILENAME,
            )
            .await?;

        tracing::info!("Proof of residence uploaded");

        // Brief delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Step 6: Submit questionnaire
        self.sumsub_client
            .submit_questionnaire_direct(&applicant_id, sumsub_testing_utils::TEST_QUESTIONNAIRE_ID)
            .await?;

        tracing::info!("Questionnaire submitted");

        // Brief delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Step 7: Request review
        self.sumsub_client.request_check(&applicant_id).await?;

        tracing::info!("Review requested");

        // Brief delay for processing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Step 8: Simulate approval (GREEN)
        self.sumsub_client
            .simulate_review_response(&applicant_id, sumsub_auth::REVIEW_ANSWER_GREEN)
            .await?;

        tracing::info!("Approval simulated");

        // Brief delay for processing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        tracing::info!(
            applicant_id = %applicant_id,
            customer_id = %customer_id,
            "Complete test applicant created and approved successfully"
        );

        Ok(applicant_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_customer::CustomerType;

    #[test]
    fn test_review_answer_parsing() {
        assert_eq!(
            "GREEN".parse::<ReviewAnswer>().unwrap(),
            ReviewAnswer::Green
        );
        assert_eq!(
            "green".parse::<ReviewAnswer>().unwrap(),
            ReviewAnswer::Green
        );
        assert_eq!("RED".parse::<ReviewAnswer>().unwrap(), ReviewAnswer::Red);
        assert_eq!("red".parse::<ReviewAnswer>().unwrap(), ReviewAnswer::Red);

        assert!("INVALID".parse::<ReviewAnswer>().is_err());
    }

    #[test]
    fn test_sumsub_verification_level_parsing() {
        assert_eq!(
            "basic-kyc-level"
                .parse::<SumsubVerificationLevel>()
                .unwrap(),
            SumsubVerificationLevel::BasicKycLevel
        );
        assert_eq!(
            "basic-kyb-level"
                .parse::<SumsubVerificationLevel>()
                .unwrap(),
            SumsubVerificationLevel::BasicKybLevel
        );

        assert!("invalid-level".parse::<SumsubVerificationLevel>().is_err());
    }

    #[test]
    fn test_customer_type_to_verification_level() {
        assert_eq!(
            SumsubVerificationLevel::from(CustomerType::Individual),
            SumsubVerificationLevel::BasicKycLevel
        );
        assert_eq!(
            SumsubVerificationLevel::from(CustomerType::PrivateCompany),
            SumsubVerificationLevel::BasicKybLevel
        );
    }

    #[test]
    fn test_sumsub_config_default() {
        let config = SumsubConfig::default();
        assert!(config.sumsub_key.is_empty());
        assert!(config.sumsub_secret.is_empty());
    }
}
