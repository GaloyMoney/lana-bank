use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ReportConfig {
    #[serde(default)]
    pub sa_creds_base64: String,
    #[serde(default)]
    pub gcp_project: String,
    #[serde(default)]
    pub gcp_location: String,
    #[serde(default)]
    pub dataform_repo: String,
    #[serde(default)]
    pub dataform_release_config: String,
}
