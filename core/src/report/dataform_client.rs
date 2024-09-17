use gcp_auth::{CustomServiceAccount, TokenProvider};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{ReportConfig, ReportError};

const SCOPES: &[&str] = &["https://www.googleapis.com/auth/cloud-platform"];

#[derive(Deserialize, Serialize)]
pub struct DataformConfig {
    service_account_creds_base64: String,
}

pub struct DataformClient {
    base_url: String,
    config: ReportConfig,
    provider: CustomServiceAccount,
}

impl DataformClient {
    pub async fn connect(config: &ReportConfig) -> Result<Self, ReportError> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD.decode(config.sa_creds_base64.as_bytes())?;
        let json = String::from_utf8(bytes)?;
        let provider = CustomServiceAccount::from_json(&json)?;

        let _ = provider.token(SCOPES).await?;

        Ok(Self {
            provider,
            config: config.clone(),
            base_url: format!(
                "https://dataform.googleapis.com/v1beta1/projects/{}/locations/{}/repositories/{}",
                config.gcp_project, config.gcp_location, config.dataform_repo
            ),
        })
    }

    pub async fn compile(&mut self) -> Result<CompilationResult, ReportError> {
        let res: serde_json::Value= self
            .make_post_request("compilationResults", serde_json::json!({
                "releaseConfig": format!("projects/{}/locations/{}/repositories/{}/releaseConfigs/{}", self.config.gcp_project, self.config.gcp_location, self.config.dataform_repo, self.config.dataform_release_config)
            }))
        .await?;
        Ok(serde_json::from_value(res)?)
    }

    pub async fn invoke(
        &mut self,
        compilation: &CompilationResult,
    ) -> Result<WorkflowInvocation, ReportError> {
        let res: serde_json::Value = self
            .make_post_request(
                "workflowInvocations",
                serde_json::json!({
                    "invocationConfig": {
                        "serviceAccount": self.config.service_account_key().client_email,
                    },
                    "compilationResult": compilation.name
                }),
            )
            .await?;
        Ok(serde_json::from_value(res)?)
    }

    async fn make_post_request<T: serde::de::DeserializeOwned>(
        &self,
        api_path: &str,
        body: serde_json::Value,
    ) -> Result<T, ReportError> {
        let body_json = serde_json::to_string(&body).expect("Couldn't serialize body");
        let client = Client::new();
        let res = client
            .post(format!("{}/{}", self.base_url, api_path))
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.provider.token(SCOPES).await?.as_str()),
            )
            .body(body_json)
            .send()
            .await?;
        dbg!(&res);
        let res: T = res.json().await?;
        Ok(res)
    }

    async fn make_get_request<T: serde::de::DeserializeOwned>(
        &self,
        api_path: &str,
    ) -> anyhow::Result<T> {
        let client = Client::new();
        let res = client
            .get(format!("{}/{}", self.base_url, api_path))
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.provider.token(SCOPES).await?.as_str()),
            )
            .send()
            .await?;
        let res: T = res.json().await?;
        Ok(res)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompilationResult {
    name: String,
    release_config: String,
    resolved_git_commit_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    Unspecified,
    Running,
    Succeeded,
    Cancelled,
    Failed,
    Canceling,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowInvocation {
    name: String,
    state: State,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UploadResult {
    pub gcs_path: String,
}
