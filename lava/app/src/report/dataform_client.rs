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
        let bytes = general_purpose::STANDARD
            .decode(config.service_account().sa_creds_base64.as_bytes())?;
        let json = String::from_utf8(bytes)?;
        let provider = CustomServiceAccount::from_json(&json)?;

        let _ = provider.token(SCOPES).await?;

        Ok(Self {
            provider,
            base_url: format!(
                "https://dataform.googleapis.com/v1beta1/projects/{}/locations/{}/repositories/{}",
                config.service_account().gcp_project,
                config.service_account().gcp_location,
                config.dataform_repo
            ),
            config: config.clone(),
        })
    }

    pub async fn compile(&mut self) -> Result<CompilationResult, ReportError> {
        let res: DataformResponse<CompilationResult> = self
            .make_post_request("compilationResults", serde_json::json!({
                "releaseConfig": format!("projects/{}/locations/{}/repositories/{}/releaseConfigs/{}", self.config.service_account().gcp_project, self.config.service_account().gcp_location, self.config.dataform_repo, self.config.dataform_release_config)
            }))
        .await?;
        match res {
            DataformResponse::Success(res) => Ok(res),
            DataformResponse::Error(err) => Err(ReportError::DataformCompilation(
                serde_json::to_string(&err).expect("Could not stringify error"),
            )),
        }
    }

    pub async fn invoke(
        &mut self,
        compilation: &CompilationResult,
    ) -> Result<WorkflowInvocation, ReportError> {
        let res: DataformResponse<WorkflowInvocation> = self
            .make_post_request(
                "workflowInvocations",
                serde_json::json!({
                    "invocationConfig": {
                        "serviceAccount": self.config.service_account().service_account_key().client_email,
                    },
                    "compilationResult": compilation.name
                }),
            )
            .await?;
        match res {
            DataformResponse::Success(res) => Ok(res),
            DataformResponse::Error(err) => Err(ReportError::DataformInvocation(
                serde_json::to_string(&err).expect("Could not stringify error"),
            )),
        }
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
        Ok(res.json().await?)
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum DataformResponse<T> {
    Success(T),
    Error(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompilationResult {
    name: String,
    release_config: String,
    resolved_git_commit_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowInvocationState {
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
    pub name: String,
    pub state: WorkflowInvocationState,
}
