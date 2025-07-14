use hmac::{Hmac, Mac};
use reqwest::{
    Client as ReqwestClient,
    header::{HeaderMap, HeaderValue},
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::primitives::CustomerId;

use super::SumsubConfig;
use super::error::ApplicantError;

const SUMSUB_BASE_URL: &str = "https://api.sumsub.com";

// Document types (testing constants)
#[cfg(feature = "sumsub-testing")]
pub const DOC_TYPE_PASSPORT: &str = "PASSPORT";
#[cfg(feature = "sumsub-testing")]
pub const DOC_TYPE_SELFIE: &str = "SELFIE";
#[cfg(feature = "sumsub-testing")]
pub const DOC_TYPE_UTILITY_BILL: &str = "UTILITY_BILL";

// Document subtypes (testing constants)
#[cfg(feature = "sumsub-testing")]
pub const DOC_SUBTYPE_FRONT_SIDE: &str = "FRONT_SIDE";
#[cfg(feature = "sumsub-testing")]
pub const DOC_SUBTYPE_BACK_SIDE: &str = "BACK_SIDE";

// Review answers (testing constants)
#[cfg(feature = "sumsub-testing")]
pub const REVIEW_ANSWER_GREEN: &str = "GREEN";
#[cfg(feature = "sumsub-testing")]
pub const REVIEW_ANSWER_RED: &str = "RED";

// Questionnaire defaults (testing constants)
#[cfg(feature = "sumsub-testing")]
const DEFAULT_QUESTIONNAIRE_SECTION: &str = "testSumsubQuestionar";
#[cfg(feature = "sumsub-testing")]
const DEFAULT_QUESTIONNAIRE_ITEM: &str = "test";
#[cfg(feature = "sumsub-testing")]
const DEFAULT_QUESTIONNAIRE_VALUE: &str = "0";

#[derive(Clone, Debug)]
pub struct SumsubClient {
    client: ReqwestClient,
    sumsub_key: String,
    sumsub_secret: String,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    description: String,
    code: u16,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum SumsubResponse<T> {
    Success(T),
    Error(ApiError),
}

#[derive(Deserialize, Debug)]
pub struct PermalinkResponse {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantDetails {
    pub id: String,
    #[serde(rename = "externalUserId")]
    pub customer_id: CustomerId,
    #[serde(default)]
    pub info: ApplicantInfo,
    #[serde(default)]
    pub fixed_info: ApplicantInfo,
    #[serde(rename = "type")]
    pub applicant_type: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicantInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub country: Option<String>,
    pub country_of_birth: Option<String>,
    pub dob: Option<String>,
    pub addresses: Option<Vec<Address>>,
    pub id_docs: Option<Vec<IdDocument>>,
}

impl ApplicantInfo {
    /// Get the applicant's first name
    pub fn first_name(&self) -> Option<&str> {
        self.first_name.as_deref()
    }

    /// Get the applicant's last name
    pub fn last_name(&self) -> Option<&str> {
        self.last_name.as_deref()
    }

    /// Get the applicant's full name as "FirstName LastName"
    pub fn full_name(&self) -> Option<String> {
        match (self.first_name(), self.last_name()) {
            (Some(first), Some(last)) => Some(format!("{first} {last}")),
            (Some(first), None) => Some(first.to_string()),
            (None, Some(last)) => Some(last.to_string()),
            (None, None) => None,
        }
    }

    /// Get the primary address (first in the list)
    pub fn primary_address(&self) -> Option<&str> {
        self.addresses
            .as_ref()?
            .first()?
            .formatted_address
            .as_deref()
    }

    /// Get nationality from country field or from identity documents
    pub fn nationality(&self) -> Option<&str> {
        // First try the country field in info
        if let Some(ref country) = self.country {
            return Some(country);
        }

        // Try country_of_birth from fixedInfo
        if let Some(ref country_of_birth) = self.country_of_birth {
            return Some(country_of_birth);
        }

        // If not found, try to get it from passport documents
        if let Some(ref id_docs) = self.id_docs {
            for doc in id_docs {
                if doc.doc_type == "PASSPORT" {
                    if let Some(ref country) = doc.country {
                        return Some(country);
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub formatted_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdDocument {
    #[serde(rename = "idDocType")]
    pub doc_type: String,
    pub country: Option<String>,
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
        }
    }

    /// Helper to create document metadata JSON
    #[cfg(feature = "sumsub-testing")]
    fn create_document_metadata(
        doc_type: &str,
        doc_sub_type: &str,
        country: Option<&str>,
    ) -> String {
        let mut json_obj = json!({ "idDocType": doc_type });

        if !doc_sub_type.is_empty() {
            json_obj["idDocSubType"] = json!(doc_sub_type);
        }

        if let Some(country_code) = country {
            json_obj["country"] = json!(country_code);
        }

        json_obj.to_string()
    }

    /// Helper to handle Sumsub API response errors
    fn handle_sumsub_error(response_text: &str, fallback_message: &str) -> ApplicantError {
        match serde_json::from_str::<SumsubResponse<serde_json::Value>>(response_text) {
            Ok(SumsubResponse::Error(ApiError { description, code })) => {
                ApplicantError::Sumsub { description, code }
            }
            _ => ApplicantError::Sumsub {
                description: format!("{fallback_message}: {response_text}"),
                code: 500,
            },
        }
    }

    /// Helper to handle API responses consistently
    async fn handle_api_response<T>(
        response: reqwest::Response,
        success_message: Option<&str>,
        error_message: &str,
    ) -> Result<T, ApplicantError>
    where
        T: serde::de::DeserializeOwned,
    {
        if response.status().is_success() {
            if let Some(msg) = success_message {
                println!("{msg}");
            }
            let parsed: SumsubResponse<T> = response.json().await?;
            match parsed {
                SumsubResponse::Success(data) => Ok(data),
                SumsubResponse::Error(ApiError { description, code }) => {
                    Err(ApplicantError::Sumsub { description, code })
                }
            }
        } else {
            let status_code = response.status().as_u16();
            let response_text = response.text().await?;
            println!("{error_message}: {response_text}");
            Err(ApplicantError::Sumsub {
                description: format!("{error_message}: {response_text}"),
                code: status_code,
            })
        }
    }

    /// Helper for simple success/error responses (no data returned)
    #[cfg(feature = "sumsub-testing")]
    async fn handle_simple_response(
        response: reqwest::Response,
        success_message: &str,
        error_message: &str,
    ) -> Result<(), ApplicantError> {
        if response.status().is_success() {
            println!("{success_message}");
            Ok(())
        } else {
            let response_text = response.text().await?;
            Err(Self::handle_sumsub_error(&response_text, error_message))
        }
    }

    pub async fn create_permalink(
        &self,
        external_user_id: CustomerId,
        level_name: &str,
    ) -> Result<PermalinkResponse, ApplicantError> {
        let method = "POST";
        let url = format!(
            "/resources/sdkIntegrations/levels/{level_name}/websdkLink?&externalUserId={external_user_id}"
        );
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url);

        let body = json!({}).to_string();
        let headers = self.get_headers(method, &url, Some(&body))?;

        let response = self
            .client
            .post(&full_url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        Self::handle_api_response(response, None, "Failed to create permalink").await
    }

    /// Get parsed applicant details with structured data
    pub async fn get_applicant_details(
        &self,
        external_user_id: CustomerId,
    ) -> Result<ApplicantDetails, ApplicantError> {
        let method = "GET";
        let url = format!("/resources/applicants/-;externalUserId={external_user_id}/one");
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url);

        let headers = self.get_headers(method, &url, None)?;
        let response = self.client.get(&full_url).headers(headers).send().await?;

        Self::handle_api_response(response, None, "Failed to get applicant details").await
    }

    fn get_headers(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<HeaderMap, ApplicantError> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
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
        timestamp: u64,
    ) -> Result<String, ApplicantError> {
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

    /// Submits a financial transaction to Sumsub for transaction monitoring
    pub async fn submit_finance_transaction(
        &self,
        customer_id: CustomerId,
        tx_id: impl Into<String>,
        tx_type: &str,
        direction: &str,
        amount: f64,
        currency_code: &str,
    ) -> Result<(), ApplicantError> {
        let method = "POST";

        // First we need to get the Sumsub applicantId for this customer
        let applicant_details = self.get_applicant_details(customer_id).await?;
        let applicant_id = &applicant_details.id;

        // Use the correct API endpoint for existing applicants
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
                "externalUserId": customer_id.to_string(),
                "fullName": ""
            }
        });

        // Make the API request
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url_path);
        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .post(&full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        // Handle the response
        if response.status().is_success() {
            Ok(())
        } else {
            let response_text = response.text().await?;
            Err(Self::handle_sumsub_error(
                &response_text,
                "Failed to post transaction",
            ))
        }
    }

    /// Creates an applicant directly via API for testing purposes
    /// This is useful for sandbox testing where you want to create an applicant
    /// without requiring a user to visit the permalink URL
    #[cfg(feature = "sumsub-testing")]
    pub async fn create_applicant(
        &self,
        external_user_id: CustomerId,
        level_name: &str,
    ) -> Result<String, ApplicantError> {
        let method = "POST";
        let url = format!("/resources/applicants?levelName={level_name}");
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url);

        let body = json!({
            "externalUserId": external_user_id.to_string(),
            "type": "individual"
        });

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url, Some(&body_str))?;

        let response = self
            .client
            .post(&full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let response_text = response.text().await?;

        match serde_json::from_str::<SumsubResponse<serde_json::Value>>(&response_text) {
            Ok(SumsubResponse::Success(applicant_data)) => {
                // Extract applicant ID from the response
                if let Some(applicant_id) = applicant_data.get("id").and_then(|id| id.as_str()) {
                    Ok(applicant_id.to_string())
                } else {
                    Err(ApplicantError::Sumsub {
                        description: "Applicant ID not found in response".to_string(),
                        code: 500,
                    })
                }
            }
            Ok(SumsubResponse::Error(ApiError { description, code })) => {
                Err(ApplicantError::Sumsub { description, code })
            }
            Err(e) => Err(ApplicantError::Serde(e)),
        }
    }

    /// Updates the fixedInfo for an applicant with basic personal data
    /// This is required before simulating approval as Sumsub needs some basic information
    #[cfg(feature = "sumsub-testing")]
    pub async fn update_applicant_info(
        &self,
        applicant_id: &str,
        first_name: &str,
        last_name: &str,
        date_of_birth: &str,    // Format: YYYY-MM-DD
        country_of_birth: &str, // 3-letter country code
    ) -> Result<(), ApplicantError> {
        let method = "PATCH";
        let url_path = format!("/resources/applicants/{applicant_id}/fixedInfo");
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url_path);

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
            .patch(&full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        Self::handle_simple_response(
            response,
            "Applicant info updated",
            "Failed to update applicant info",
        )
        .await
    }

    /// Uploads a document image for an applicant
    /// This method handles the multipart form data upload required for document images
    #[cfg(feature = "sumsub-testing")]
    pub async fn upload_document(
        &self,
        applicant_id: &str,
        doc_type: &str,        // e.g., "PASSPORT", "SELFIE", "ID_CARD"
        doc_sub_type: &str,    // e.g., "FRONT_SIDE", "BACK_SIDE", or empty for single-sided docs
        country: Option<&str>, // 3-letter country code (e.g., "USA", "DEU") - required for most docs
        image_data: Vec<u8>,   // Image file data
        filename: &str,        // e.g., "passport.jpg"
    ) -> Result<(), ApplicantError> {
        // Use manual multipart construction directly for reliable HMAC signature calculation
        self.upload_document_with_manual_multipart(
            applicant_id,
            doc_type,
            doc_sub_type,
            country,
            image_data,
            filename,
        )
        .await
    }

    /// Uploads document with manual multipart body construction for proper HMAC signature calculation
    #[cfg(feature = "sumsub-testing")]
    async fn upload_document_with_manual_multipart(
        &self,
        applicant_id: &str,
        doc_type: &str,
        doc_sub_type: &str,
        country: Option<&str>,
        image_data: Vec<u8>,
        filename: &str,
    ) -> Result<(), ApplicantError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/info/idDoc");
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url_path);

        let metadata = Self::create_document_metadata(doc_type, doc_sub_type, country);

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

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
            .post(&full_url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        Self::handle_simple_response(
            response,
            &format!("Document uploaded successfully: {doc_type} {doc_sub_type}"),
            "Document upload failed",
        )
        .await
    }

    /// Requests a check/review for an applicant
    /// This moves the applicant to "pending" status for review
    #[cfg(feature = "sumsub-testing")]
    pub async fn request_check(&self, applicant_id: &str) -> Result<(), ApplicantError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/status/pending");
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url_path);

        let body = json!({}).to_string();
        let headers = self.get_headers(method, &url_path, Some(&body))?;

        let response = self
            .client
            .post(&full_url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        Self::handle_simple_response(
            response,
            "Review requested successfully",
            "Failed to request check",
        )
        .await
    }

    /// Simulates a review response in sandbox mode (GREEN for approved, RED for rejected)
    /// This is only available in sandbox environments for testing purposes
    #[cfg(feature = "sumsub-testing")]
    pub async fn simulate_review_response(
        &self,
        applicant_id: &str,
        review_answer: &str, // "GREEN" or "RED"
    ) -> Result<(), ApplicantError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/status/testCompleted");
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url_path);

        let body = if review_answer == REVIEW_ANSWER_GREEN {
            json!({
                "reviewAnswer": REVIEW_ANSWER_GREEN,
                "rejectLabels": []
            })
        } else {
            json!({
                "reviewAnswer": REVIEW_ANSWER_RED,
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
            .post(&full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        // Handle the response
        if response.status().is_success() {
            Ok(())
        } else {
            // Extract error details if available
            let response_text = response.text().await?;
            match serde_json::from_str::<SumsubResponse<serde_json::Value>>(&response_text) {
                Ok(SumsubResponse::Error(ApiError { description, code })) => {
                    Err(ApplicantError::Sumsub { description, code })
                }
                _ => Err(ApplicantError::Sumsub {
                    description: format!("Failed to simulate review: {response_text}"),
                    code: 500,
                }),
            }
        }
    }

    /// Submits a questionnaire directly to an applicant
    #[cfg(feature = "sumsub-testing")]
    pub async fn submit_questionnaire_direct(
        &self,
        applicant_id: &str,
        questionnaire_id: &str,
    ) -> Result<(), ApplicantError> {
        let method = "POST";
        let url_path = format!("/resources/applicants/{applicant_id}/questionnaires");
        let full_url = format!("{SUMSUB_BASE_URL}{url_path}");

        // Create a basic questionnaire submission based on the v1_onboarding questionnaire
        let body = json!({
            "id": questionnaire_id,
            "sections": {
                DEFAULT_QUESTIONNAIRE_SECTION: {
                    "items": {
                        DEFAULT_QUESTIONNAIRE_ITEM: {
                            "value": DEFAULT_QUESTIONNAIRE_VALUE
                        }
                    }
                }
            }
        });

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .post(&full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        if response.status().is_success() {
            println!("Questionnaire submitted successfully");
            Ok(())
        } else {
            let response_text = response.text().await?;
            println!("Questionnaire submission response: {response_text}");

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
    ) -> Result<(), ApplicantError> {
        let method = "PATCH";
        let url_path =
            format!("/resources/applicants/{applicant_id}/questionnaires/{questionnaire_id}");
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url_path);

        let body = json!({
            "sections": {
                DEFAULT_QUESTIONNAIRE_SECTION: {
                    "items": {
                        DEFAULT_QUESTIONNAIRE_ITEM: {
                            "value": DEFAULT_QUESTIONNAIRE_VALUE
                        }
                    }
                }
            }
        });

        let body_str = body.to_string();
        let headers = self.get_headers(method, &url_path, Some(&body_str))?;

        let response = self
            .client
            .patch(&full_url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        if response.status().is_success() {
            println!("Questionnaire updated successfully (alternative approach)");
            Ok(())
        } else {
            let response_text = response.text().await?;
            Err(Self::handle_sumsub_error(
                &response_text,
                "Failed to submit questionnaire via both methods",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_applicant_info_methods() {
        let applicant_info = ApplicantInfo {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            country: Some("USA".to_string()),
            country_of_birth: None,
            dob: None,
            addresses: None,
            id_docs: None,
        };

        assert_eq!(applicant_info.first_name(), Some("John"));
        assert_eq!(applicant_info.last_name(), Some("Doe"));
        assert_eq!(applicant_info.full_name(), Some("John Doe".to_string()));
        assert_eq!(applicant_info.nationality(), Some("USA"));
    }
}
