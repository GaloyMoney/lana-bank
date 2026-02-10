#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;
mod wire;

#[cfg(feature = "sumsub-testing")]
pub mod testing_utils;

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{
    Client as ReqwestClient, Url,
    header::{HeaderMap, HeaderValue},
};
use serde_json::json;
use sha2::Sha256;

pub use config::SumsubConfig;
pub use error::SumsubError;
pub use wire::*;

const SUMSUB_BASE_URL: &str = "https://api.sumsub.com";

/// Sumsub API client
#[derive(Clone, Debug)]
pub struct SumsubClient {
    client: ReqwestClient,
    sumsub_key: String,
    sumsub_secret: String,
    base_url: Url,
}

impl SumsubClient {
    pub fn new(config: &SumsubConfig) -> Self {
        Self {
            client: ReqwestClient::builder()
                .use_rustls_tls()
                .build()
                .expect("should always build SumsubClient"),
            sumsub_key: config.sumsub_key.clone(),
            sumsub_secret: config.sumsub_secret.clone(),
            base_url: Url::parse(SUMSUB_BASE_URL).expect("SUMSUB_BASE_URL should be a valid URL"),
        }
    }

    /// Create a permalink for SDK integration
    pub async fn create_permalink<T>(
        &self,
        external_user_id: T,
        level_name: &str,
    ) -> Result<PermalinkResponse, SumsubError>
    where
        T: std::fmt::Display,
    {
        let method = "POST";
        let url = format!(
            "/resources/sdkIntegrations/levels/{level_name}/websdkLink?&externalUserId={external_user_id}"
        );
        let full_url = self.base_url.join(&url).expect("valid URL");

        let body = json!({}).to_string();
        let headers = self.get_headers(method, &url, Some(&body))?;

        let response = self
            .client
            .post(full_url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        self.handle_api_response(response, "Failed to create permalink")
            .await
    }

    /// Get applicant details by external user ID
    pub async fn get_applicant_details<T>(
        &self,
        external_user_id: T,
    ) -> Result<ApplicantDetails<T>, SumsubError>
    where
        T: std::fmt::Display + serde::de::DeserializeOwned,
    {
        let method = "GET";
        let url = format!("/resources/applicants/-;externalUserId={external_user_id}/one");
        let full_url = self.base_url.join(&url).expect("valid URL");

        let headers = self.get_headers(method, &url, None)?;
        let response = self.client.get(full_url).headers(headers).send().await?;

        self.handle_api_response(response, "Failed to get applicant details")
            .await
    }

    /// Submit a financial transaction for monitoring.
    ///
    /// This operation is idempotent: if a transaction with the given `tx_id` already exists,
    /// the duplicate error is treated as success since the goal (transaction in Sumsub) is achieved.
    pub async fn submit_finance_transaction<T>(
        &self,
        external_user_id: T,
        tx_id: impl Into<String>,
        tx_type: &str,
        direction: &str,
        amount: f64,
        currency_code: &str,
    ) -> Result<(), SumsubError>
    where
        T: std::fmt::Display + Clone + serde::de::DeserializeOwned,
    {
        // First get the applicant details to obtain the internal applicant ID
        let applicant_details = self.get_applicant_details(external_user_id.clone()).await?;
        let applicant_id = &applicant_details.id;

        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/kyt/txns/-/data");
        let tx_id = tx_id.into();

        // Current timestamp for the request
        let now = chrono::Utc::now();
        let date_format = now.format("%Y-%m-%d %H:%M:%S+0000").to_string();

        // Build the request body
        let body = json!({
            "txnId": tx_id,
            "type": "finance",
            "txnDate": date_format,
            "info": {
                "type": tx_type,
                "direction": direction,
                "amount": amount,
                "currencyCode": currency_code,
                "currencyType": "fiat",
                "paymentDetails": ""
            },
            "applicant": {
                "type": "individual",
                "externalUserId": external_user_id.to_string(),
                "fullName": ""
            }
        });

        let full_url = self.base_url.join(&url_path).expect("valid URL");
        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .post(full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            tracing::info!(
                tx_id = %tx_id,
                tx_type = %tx_type,
                direction = %direction,
                amount = %amount,
                currency = %currency_code,
                customer_id = %external_user_id,
                "Submitted finance transaction to Sumsub"
            );
            Ok(())
        } else if status == reqwest::StatusCode::CONFLICT {
            // HTTP 409 Conflict with "Entity already exists" indicates duplicate transaction
            // Treat as success for idempotency - see https://docs.sumsub.com/reference/submit-transaction-for-existing-applicant
            tracing::warn!(
                tx_id = %tx_id,
                customer_id = %external_user_id,
                "Transaction already exists in Sumsub, skipping (idempotent)"
            );
            Ok(())
        } else {
            let response_text = response.text().await?;
            // Check for "Entity already exists" error in response body (Sumsub's duplicate error format)
            if response_text.contains("Entity already exists") {
                tracing::warn!(
                    tx_id = %tx_id,
                    customer_id = %external_user_id,
                    "Transaction already exists in Sumsub, skipping (idempotent)"
                );
                return Ok(());
            }
            Err(self.handle_sumsub_error(&response_text, "Failed to post transaction"))
        }
    }

    // Testing methods (only available with sumsub-testing feature)
    #[cfg(feature = "sumsub-testing")]
    pub async fn create_applicant<T>(
        &self,
        external_user_id: T,
        level_name: &str,
    ) -> Result<String, SumsubError>
    where
        T: std::fmt::Display,
    {
        let method = "POST";
        let url = format!("/resources/applicants?levelName={level_name}");
        let full_url = self.base_url.join(&url).expect("valid URL");

        let body = json!({
            "externalUserId": external_user_id.to_string(),
            "type": "individual"
        });

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url, Some(&body_str))?;

        let response = self
            .client
            .post(full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let response_text = response.text().await?;

        // Parse as ApiError first to avoid the untagged enum issue where
        // SumsubResponse::Success(serde_json::Value) swallows any valid JSON
        if let Ok(api_error) = serde_json::from_str::<wire::ApiError>(&response_text) {
            if api_error.error_name.is_some() || api_error.error_code.is_some() {
                Self::record_sumsub_error_metadata(&api_error);
                return Err(SumsubError::ApiError {
                    description: api_error.description,
                    code: api_error.code,
                });
            }
        }

        match serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(applicant_data) => {
                if let Some(applicant_id) = applicant_data.get("id").and_then(|id| id.as_str()) {
                    Ok(applicant_id.to_string())
                } else {
                    Err(SumsubError::InvalidResponse(
                        "Applicant ID not found in response".to_string(),
                    ))
                }
            }
            Err(e) => Err(SumsubError::JsonFormat(e)),
        }
    }

    #[cfg(feature = "sumsub-testing")]
    pub async fn update_applicant_info(
        &self,
        applicant_id: &str,
        first_name: &str,
        last_name: &str,
        date_of_birth: &str,    // Format: YYYY-MM-DD
        country_of_birth: &str, // 3-letter country code
    ) -> Result<(), SumsubError> {
        let method = "PATCH";
        let url_path = format!("/resources/applicants/{applicant_id}/fixedInfo");
        let full_url = self.base_url.join(&url_path).expect("valid URL");

        let body = json!({
            "firstName": first_name,
            "lastName": last_name,
            "dob": date_of_birth,
            "countryOfBirth": country_of_birth
        });

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .patch(full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        self.handle_simple_response(response, "Failed to update applicant info")
            .await
    }

    #[cfg(feature = "sumsub-testing")]
    pub async fn simulate_review_response(
        &self,
        applicant_id: &str,
        review_answer: &str, // "GREEN" or "RED"
    ) -> Result<(), SumsubError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/status/testCompleted");
        let full_url = self.base_url.join(&url_path).expect("valid URL");

        let body = if review_answer == wire::testing::REVIEW_ANSWER_GREEN {
            json!({
                "reviewAnswer": wire::testing::REVIEW_ANSWER_GREEN,
                "rejectLabels": []
            })
        } else {
            json!({
                "reviewAnswer": wire::testing::REVIEW_ANSWER_RED,
                "rejectLabels": ["UNSATISFACTORY_PHOTOS"],
                "reviewRejectType": "RETRY",
                "clientComment": "Test rejection for automated testing",
                "moderationComment": "This is a simulated rejection for testing purposes"
            })
        };

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .post(full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let response_text = response.text().await?;
            match serde_json::from_str::<wire::ApiError>(&response_text) {
                Ok(api_error) => {
                    Self::record_sumsub_error_metadata(&api_error);
                    Err(SumsubError::ApiError {
                        description: api_error.description,
                        code: api_error.code,
                    })
                }
                _ => Err(SumsubError::InvalidResponse(format!(
                    "Failed to simulate review: {response_text}"
                ))),
            }
        }
    }

    /// Uploads document with manual multipart body construction for proper HMAC signature calculation
    #[cfg(feature = "sumsub-testing")]
    pub async fn upload_document(
        &self,
        applicant_id: &str,
        doc_type: &str,
        doc_sub_type: &str,
        country: Option<&str>,
        image_data: Vec<u8>,
        filename: &str,
    ) -> Result<(), SumsubError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/info/idDoc");
        let full_url = self.base_url.join(&url_path).expect("valid URL");

        let metadata = Self::create_document_metadata(doc_type, doc_sub_type, country);

        let timestamp = Utc::now().timestamp();

        // Manually construct multipart body for signature calculation
        let boundary = format!("----formdata-reqwest-{timestamp}");
        let mut body = Vec::new();

        // Add metadata field
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"metadata\"\r\n\r\n");
        body.extend_from_slice(metadata.as_bytes());
        body.extend_from_slice(b"\r\n");

        // Add file field
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"content\"; filename=\"{filename}\"\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(&image_data);
        body.extend_from_slice(b"\r\n");

        // Add closing boundary
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        // Calculate signature with the manual multipart body
        let signature = {
            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(self.sumsub_secret.as_bytes())
                .expect("HMAC can take key of any size");
            mac.update(timestamp.to_string().as_bytes());
            mac.update(method.as_bytes());
            mac.update(url_path.as_bytes());
            mac.update(&body);
            hex::encode(mac.finalize().into_bytes())
        };

        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert(
            "Content-Type",
            HeaderValue::from_str(&format!("multipart/form-data; boundary={boundary}"))?,
        );
        headers.insert(
            "X-App-Token",
            HeaderValue::from_str(&self.sumsub_key).expect("Invalid sumsub key"),
        );
        headers.insert(
            "X-App-Access-Ts",
            HeaderValue::from_str(&timestamp.to_string()).expect("Invalid timestamp"),
        );
        headers.insert("X-App-Access-Sig", HeaderValue::from_str(&signature)?);

        let response = self
            .client
            .post(full_url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        self.handle_simple_response(response, "Failed to upload document")
            .await
    }

    /// Helper method to create document metadata
    #[cfg(feature = "sumsub-testing")]
    fn create_document_metadata(
        doc_type: &str,
        doc_sub_type: &str,
        country: Option<&str>,
    ) -> String {
        if doc_sub_type.is_empty() {
            json!({
                "idDocType": doc_type,
                "country": country.unwrap_or("DEU")
            })
            .to_string()
        } else {
            json!({
                "idDocType": doc_type,
                "idDocSubType": doc_sub_type,
                "country": country.unwrap_or("DEU")
            })
            .to_string()
        }
    }

    #[cfg(feature = "sumsub-testing")]
    pub async fn submit_questionnaire_direct(
        &self,
        applicant_id: &str,
        questionnaire_id: &str,
    ) -> Result<(), SumsubError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/questionnaires");
        let full_url = self.base_url.join(&url_path).expect("valid URL");

        // Create a basic questionnaire submission based on the v1_onboarding questionnaire
        let body = json!({
            "id": questionnaire_id,
            "sections": {
                wire::testing::DEFAULT_QUESTIONNAIRE_SECTION: {
                    "items": {
                        wire::testing::DEFAULT_QUESTIONNAIRE_ITEM: {
                            "value": wire::testing::DEFAULT_QUESTIONNAIRE_VALUE
                        }
                    }
                }
            }
        });

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .post(full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Questionnaire submitted successfully");
            Ok(())
        } else {
            let response_text = response.text().await?;
            tracing::info!("Questionnaire submission response: {response_text}");

            // Try alternative approach: add questionnaire data directly to applicant info
            self.update_applicant_questionnaire(applicant_id, questionnaire_id)
                .await
        }
    }

    /// Alternative approach: Update applicant with questionnaire data
    #[cfg(feature = "sumsub-testing")]
    async fn update_applicant_questionnaire(
        &self,
        applicant_id: &str,
        questionnaire_id: &str,
    ) -> Result<(), SumsubError> {
        let method = "PATCH";
        let url_path =
            format!("/resources/applicants/{applicant_id}/questionnaires/{questionnaire_id}");
        let full_url = self.base_url.join(&url_path).expect("valid URL");

        let body = json!({
            "sections": {
                wire::testing::DEFAULT_QUESTIONNAIRE_SECTION: {
                    "items": {
                        wire::testing::DEFAULT_QUESTIONNAIRE_ITEM: {
                            "value": wire::testing::DEFAULT_QUESTIONNAIRE_VALUE
                        }
                    }
                }
            }
        });

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .patch(full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Questionnaire updated successfully (alternative approach)");
            Ok(())
        } else {
            let response_text = response.text().await?;
            Err(self.handle_sumsub_error(
                &response_text,
                "Failed to submit questionnaire via both methods",
            ))
        }
    }

    #[cfg(feature = "sumsub-testing")]
    pub async fn request_check(&self, applicant_id: &str) -> Result<(), SumsubError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/status/pending");
        let full_url = self.base_url.join(&url_path).expect("valid URL");

        let body = json!({}).to_string();
        let headers = self.get_headers(method, &url_path, Some(&body))?;

        let response = self
            .client
            .post(full_url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        self.handle_simple_response(response, "Failed to request check")
            .await
    }

    // Private helper methods
    fn get_headers(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<HeaderMap, SumsubError> {
        let timestamp = Utc::now().timestamp();
        let signature = self.sign(method, url, body, timestamp)?;

        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert(
            "X-App-Token",
            HeaderValue::from_str(&self.sumsub_key).expect("Invalid sumsub key"),
        );
        headers.insert(
            "X-App-Access-Ts",
            HeaderValue::from_str(&timestamp.to_string()).expect("Invalid timestamp"),
        );
        headers.insert("X-App-Access-Sig", HeaderValue::from_str(&signature)?);

        Ok(headers)
    }

    fn sign(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        timestamp: i64,
    ) -> Result<String, SumsubError> {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.sumsub_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(timestamp.to_string().as_bytes());
        mac.update(method.as_bytes());
        mac.update(url.as_bytes());
        if let Some(body) = body {
            mac.update(body.as_bytes());
        }
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    async fn handle_api_response<T>(
        &self,
        response: reqwest::Response,
        error_message: &str,
    ) -> Result<T, SumsubError>
    where
        T: serde::de::DeserializeOwned,
    {
        if response.status().is_success() {
            let parsed: wire::SumsubResponse<T> = response.json().await?;
            match parsed {
                wire::SumsubResponse::Success(data) => Ok(data),
                wire::SumsubResponse::Error(api_error) => {
                    Self::record_sumsub_error_metadata(&api_error);
                    Err(SumsubError::ApiError {
                        description: api_error.description,
                        code: api_error.code,
                    })
                }
            }
        } else {
            let status_code = response.status().as_u16();
            let response_text = response.text().await?;
            match serde_json::from_str::<wire::ApiError>(&response_text) {
                Ok(api_error) => {
                    Self::record_sumsub_error_metadata(&api_error);
                    let description = format!(
                        "{error_message}: {}",
                        api_error
                            .error_name
                            .as_deref()
                            .unwrap_or(&api_error.description)
                    );
                    Err(SumsubError::ApiError {
                        description,
                        code: status_code,
                    })
                }
                Err(_) => Err(SumsubError::ApiError {
                    description: format!("{error_message}: {response_text}"),
                    code: status_code,
                }),
            }
        }
    }

    #[cfg(feature = "sumsub-testing")]
    async fn handle_simple_response(
        &self,
        response: reqwest::Response,
        error_message: &str,
    ) -> Result<(), SumsubError> {
        if response.status().is_success() {
            Ok(())
        } else {
            let response_text = response.text().await?;
            Err(self.handle_sumsub_error(&response_text, error_message))
        }
    }

    fn handle_sumsub_error(&self, response_text: &str, fallback_message: &str) -> SumsubError {
        match serde_json::from_str::<wire::ApiError>(response_text) {
            Ok(api_error) => {
                Self::record_sumsub_error_metadata(&api_error);
                let description = format!(
                    "{fallback_message}: {}",
                    api_error
                        .error_name
                        .as_deref()
                        .unwrap_or(&api_error.description)
                );
                SumsubError::ApiError {
                    description,
                    code: api_error.code,
                }
            }
            _ => SumsubError::ApiError {
                description: format!("{fallback_message}: {response_text}"),
                code: 500,
            },
        }
    }

    fn record_sumsub_error_metadata(api_error: &wire::ApiError) {
        tracing::info!(
            sumsub.correlation_id = api_error.correlation_id.as_deref().unwrap_or(""),
            sumsub.error_name = api_error.error_name.as_deref().unwrap_or(""),
            sumsub.error_code = api_error.error_code.unwrap_or(0),
            sumsub.description = %api_error.description,
            "Sumsub API error details"
        );
    }
}
