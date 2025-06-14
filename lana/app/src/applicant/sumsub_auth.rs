use hmac::{Hmac, Mac};
use reqwest::{
    Client as ReqwestClient,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::primitives::CustomerId;

use super::SumsubConfig;
use super::error::ApplicantError;

const SUMSUB_BASE_URL: &str = "https://api.sumsub.com";

#[derive(Clone, Debug)]
pub struct SumsubClient {
    client: ReqwestClient,
    pub sumsub_key: String,
    pub sumsub_secret: String,
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
#[serde(rename_all = "camelCase")]
pub struct AccessTokenResponse {
    #[serde(rename = "userId")]
    pub customer_id: String,
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct PermalinkResponse {
    pub url: String,
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

        match response.json().await? {
            SumsubResponse::Success(PermalinkResponse { url }) => Ok(PermalinkResponse { url }),
            SumsubResponse::Error(ApiError { description, code }) => {
                Err(ApplicantError::Sumsub { description, code })
            }
        }
    }

    pub async fn get_applicant_details(
        &self,
        external_user_id: CustomerId,
    ) -> Result<String, ApplicantError> {
        let method = "GET";
        let url = format!(
            "/resources/applicants/-;externalUserId={}/one",
            external_user_id
        );
        let full_url = format!("{}{}", SUMSUB_BASE_URL, &url);

        let headers = self.get_headers(method, &url, None)?;
        let response = self.client.get(&full_url).headers(headers).send().await?;

        let response_text = response.text().await?;

        match serde_json::from_str::<SumsubResponse<serde_json::Value>>(&response_text) {
            Ok(SumsubResponse::Success(_)) => Ok(response_text),
            Ok(SumsubResponse::Error(ApiError { description, code })) => {
                Err(ApplicantError::Sumsub { description, code })
            }
            Err(e) => Err(ApplicantError::Serde(e)),
        }
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
        external_user_id: CustomerId,
        tx_id: impl Into<String>,
        tx_type: &str,
        direction: &str,
        amount: f64,
        currency_code: &str,
    ) -> Result<(), ApplicantError> {
        let method = "POST";

        // First we need to get the Sumsub applicantId for this customer
        let applicant_details = self.get_applicant_details(external_user_id).await?;

        // Parse the JSON response to extract the applicantId
        let applicant_json: serde_json::Value = serde_json::from_str(&applicant_details)?;
        let applicant_id = applicant_json["id"]
            .as_str()
            .ok_or_else(|| ApplicantError::Sumsub {
                description: "Applicant ID not found in the response".to_string(),
                code: 500,
            })?;

        // Use the correct API endpoint for existing applicants
        let url_path = format!("/resources/applicants/{}/kyt/txns/-/data", applicant_id);
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
            // Extract error details if available
            let response_text = response.text().await?;
            match serde_json::from_str::<SumsubResponse<serde_json::Value>>(&response_text) {
                Ok(SumsubResponse::Error(ApiError { description, code })) => {
                    Err(ApplicantError::Sumsub { description, code })
                }
                _ => Err(ApplicantError::Sumsub {
                    description: format!("Failed to post transaction: {}", response_text),
                    code: 500,
                }),
            }
        }
    }
}
