#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
mod repo;

#[cfg(feature = "sumsub-testing")]
pub use sumsub::testing_utils as sumsub_testing_utils;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerEvent, CustomerId, Customers};
use outbox::OutboxEventMarker;

use error::ApplicantError;
pub use sumsub::SumsubConfig;

use repo::ApplicantRepo;
pub use sumsub::{ApplicantInfo, PermalinkResponse, SumsubClient};

#[cfg(feature = "graphql")]
use async_graphql::*;

es_entity::entity_id!(ApplicantId);

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
    #[serde(rename = "applicantPending")]
    #[serde(rename_all = "camelCase")]
    ApplicantPending {
        applicant_id: String,
        sandbox_mode: Option<bool>,
    },
    #[serde(rename = "applicantPersonalInfoChanged")]
    #[serde(rename_all = "camelCase")]
    ApplicantPersonalInfoChanged {
        applicant_id: String,
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

pub struct Applicants<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    authz: Perms,
    sumsub_client: SumsubClient,
    repo: ApplicantRepo,
    customers: Customers<Perms, E>,
}

impl<Perms, E> Clone for Applicants<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            sumsub_client: self.sumsub_client.clone(),
            repo: self.repo.clone(),
            customers: self.customers.clone(),
        }
    }
}

impl<Perms, E> Applicants<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_customer::CustomerObject>,
{
    pub fn new(
        pool: &PgPool,
        config: &SumsubConfig,
        authz: &Perms,
        customers: &Customers<Perms, E>,
    ) -> Self {
        let sumsub_client = SumsubClient::new(config);

        Self {
            authz: authz.clone(),
            repo: ApplicantRepo::new(pool),
            sumsub_client,
            customers: customers.clone(),
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

        let mut db = self.repo.begin_op().await?;

        match self.process_payload(&mut db, payload).await {
            Ok(_) => (),
            // Silenty ignoring these errors instead of returning, 
            // this prevents sumsub from retrying for these unhandled cases
            Err(ApplicantError::UnhandledCallbackType) => (),
            Err(ApplicantError::UnhandledLevelType) => (),
            Err(e) => return Err(e),
        }

        db.commit().await?;

        Ok(())
    }

    #[instrument(
        name = "applicant.process_payload",
        skip(self, db),
        fields(ignore_for_sandbox = false, callback_type = tracing::field::Empty, sandbox_mode = tracing::field::Empty, applicant_id = tracing::field::Empty, level = tracing::field::Empty, customer_id = tracing::field::Empty),
        err
    )]
    async fn process_payload(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payload: serde_json::Value,
    ) -> Result<(), ApplicantError> {
        match serde_json::from_value(payload.clone())? {
            SumsubCallbackPayload::ApplicantCreated {
                external_user_id,
                applicant_id,
                level_name,
                sandbox_mode,
                ..
            } => {
                tracing::Span::current().record("callback_type", "ApplicantCreated");
                tracing::Span::current().record("sandbox_mode", sandbox_mode.unwrap_or(false));
                tracing::Span::current().record("applicant_id", applicant_id.as_str());
                tracing::Span::current().record("level", level_name.as_str());
                tracing::Span::current()
                    .record("customer_id", external_user_id.to_string().as_str());
                let res = self
                    .customers
                    .start_kyc(db, external_user_id, applicant_id)
                    .await;

                match res {
                    Ok(_) => (),
                    Err(e) if e.was_not_found() && sandbox_mode.unwrap_or(false) => {
                        tracing::Span::current().record("ignore_for_sandbox", true);
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
                level_name,
                sandbox_mode,
                ..
            } => {
                tracing::Span::current().record("callback_type", "ApplicantReviewed.Red");
                tracing::Span::current().record("sandbox_mode", sandbox_mode.unwrap_or(false));
                tracing::Span::current().record("applicant_id", applicant_id.as_str());
                tracing::Span::current().record("level", level_name.as_str());
                tracing::Span::current()
                    .record("customer_id", external_user_id.to_string().as_str());
                let res = self
                    .customers
                    .decline_kyc(db, external_user_id, applicant_id)
                    .await;

                match res {
                    Ok(_) => (),
                    Err(e) if e.was_not_found() && sandbox_mode.unwrap_or(false) => {
                        tracing::Span::current().record("ignore_for_sandbox", true);
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
                level_name,
                sandbox_mode,
                ..
            } => {
                tracing::Span::current().record("callback_type", "ApplicantReviewed.Green");
                tracing::Span::current().record("sandbox_mode", sandbox_mode.unwrap_or(false));
                tracing::Span::current().record("applicant_id", applicant_id.as_str());
                tracing::Span::current().record("level", level_name.as_str());
                tracing::Span::current()
                    .record("customer_id", external_user_id.to_string().as_str());
                // Try to parse the level name, will return error for unrecognized values
                match level_name.parse::<SumsubVerificationLevel>() {
                    Ok(_) => {} // Level is valid, continue
                    Err(_) => {
                        return Err(ApplicantError::UnhandledLevelType);
                    }
                };

                let res = self
                    .customers
                    .approve_kyc(db, external_user_id, applicant_id)
                    .await;

                match res {
                    Ok(_) => (),
                    Err(e) if e.was_not_found() && sandbox_mode.unwrap_or(false) => {
                        tracing::Span::current().record("ignore_for_sandbox", true);
                        return Ok(());
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            SumsubCallbackPayload::ApplicantPending {
                applicant_id,
                sandbox_mode,
                ..
            } => {
                // No-op: we don't need to process pending applicants
                tracing::Span::current().record("callback_type", "ApplicantPending");
                tracing::Span::current().record("sandbox_mode", sandbox_mode.unwrap_or(false));
                tracing::Span::current().record("applicant_id", applicant_id.as_str());
            }
            SumsubCallbackPayload::ApplicantPersonalInfoChanged {
                applicant_id,
                sandbox_mode,
                ..
            } => {
                // No-op: we don't need to process personal info changes
                tracing::Span::current().record("callback_type", "ApplicantPersonalInfoChanged");
                tracing::Span::current().record("sandbox_mode", sandbox_mode.unwrap_or(false));
                tracing::Span::current().record("applicant_id", applicant_id.as_str());
            }
            SumsubCallbackPayload::Unknown => {
                tracing::Span::current().record("callback_type", "Unknown");
                return Err(ApplicantError::UnhandledCallbackType);
            }
        }
        Ok(())
    }

    #[instrument(name = "applicant.create_permalink", skip(self))]
    pub async fn create_permalink(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<PermalinkResponse, ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        self.authz
            .enforce_permission(
                sub,
                core_customer::CustomerObject::customer(customer_id),
                core_customer::CoreCustomerAction::CUSTOMER_READ,
            )
            .await?;

        let customer = self.customers.find_by_id_without_audit(customer_id).await?;
        let level: SumsubVerificationLevel = customer.customer_type.into();

        Ok(self
            .sumsub_client
            .create_permalink(customer_id, &level.to_string())
            .await?)
    }

    #[instrument(name = "applicant.get_applicant_info_without_audit", skip(self))]
    pub async fn get_applicant_info_without_audit(
        &self,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<ApplicantInfo, ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        // will return error if customer not found
        self.customers.find_by_id_without_audit(customer_id).await?;

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
    ) -> Result<String, ApplicantError> {
        let customer_id: CustomerId = customer_id.into();

        // will return error if customer not found
        let customer = self.customers.find_by_id_without_audit(customer_id).await?;
        let level: SumsubVerificationLevel = customer.customer_type.into();

        tracing::info!(
            customer_id = %customer_id,
            "Creating complete test applicant with full KYC flow"
        );

        // Step 1: Create applicant via API
        let applicant_id = self
            .sumsub_client
            .create_applicant(customer_id, &level.to_string())
            .await?;

        tracing::info!(applicant_id = %applicant_id, "Applicant created");

        // Step 2: Update applicant personal information
        self.sumsub_client
            .update_applicant_info(
                &applicant_id,
                sumsub::testing::TEST_FIRST_NAME,
                sumsub::testing::TEST_LAST_NAME,
                sumsub::testing::TEST_DATE_OF_BIRTH,
                sumsub::testing::TEST_COUNTRY_CODE,
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
        .map_err(|e| {
            ApplicantError::SumsubError(sumsub::SumsubError::ApiError {
                description: format!("Failed to load passport image: {e}"),
                code: 500,
            })
        })?;

        self.sumsub_client
            .upload_document(
                &applicant_id,
                sumsub::testing::DOC_TYPE_PASSPORT,
                sumsub::testing::DOC_SUBTYPE_FRONT_SIDE,
                Some(sumsub::testing::TEST_COUNTRY_CODE),
                passport_image.clone(),
                sumsub_testing_utils::PASSPORT_FILENAME,
            )
            .await?;

        // Brief delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        self.sumsub_client
            .upload_document(
                &applicant_id,
                sumsub::testing::DOC_TYPE_PASSPORT,
                sumsub::testing::DOC_SUBTYPE_BACK_SIDE,
                Some(sumsub::testing::TEST_COUNTRY_CODE),
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
                sumsub::testing::DOC_TYPE_SELFIE,
                "",
                Some(sumsub::testing::TEST_COUNTRY_CODE),
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
        .map_err(|e| {
            ApplicantError::SumsubError(sumsub::SumsubError::ApiError {
                description: format!("Failed to load proof of residence image: {e}"),
                code: 500,
            })
        })?;

        self.sumsub_client
            .upload_document(
                &applicant_id,
                sumsub::testing::DOC_TYPE_UTILITY_BILL,
                "",
                Some(sumsub::testing::TEST_COUNTRY_CODE),
                poa_image,
                sumsub_testing_utils::POA_FILENAME,
            )
            .await?;

        tracing::info!("Proof of residence uploaded");

        // Brief delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Step 6: Submit questionnaire
        self.sumsub_client
            .submit_questionnaire_direct(&applicant_id, sumsub::testing::TEST_QUESTIONNAIRE_ID)
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
            .simulate_review_response(&applicant_id, sumsub::testing::REVIEW_ANSWER_GREEN)
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
