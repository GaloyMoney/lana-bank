mod error;

use error::ServiceAccountError;
use gcp_bigquery_client::yup_oauth2::ServiceAccountKey;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceAccountConfig {
    #[serde(skip)]
    pub gcp_project: String,
    #[serde(skip)]
    pub sa_creds_base64: String,
    #[serde(skip)]
    service_account_key: Option<ServiceAccountKey>,

    #[serde(default = "default_gcp_location")]
    pub gcp_location: String,
}

impl Default for ServiceAccountConfig {
    fn default() -> Self {
        Self {
            gcp_project: "".to_string(),
            sa_creds_base64: "".to_string(),
            service_account_key: None,
            gcp_location: default_gcp_location(),
        }
    }
}

impl ServiceAccountConfig {
    pub fn set_sa_creds_base64(
        mut self,
        sa_creds_base64: String,
    ) -> Result<Self, ServiceAccountError> {
        self.sa_creds_base64 = sa_creds_base64;

        let creds = self.get_json_creds()?;
        let service_account_key = serde_json::from_str::<ServiceAccountKey>(&creds)?;
        std::env::set_var("SERVICE_ACCOUNT_JSON", creds);

        self.gcp_project = service_account_key
            .project_id
            .clone()
            .ok_or(ServiceAccountError::GCPProjectIdMissing)?;
        self.service_account_key = Some(service_account_key);

        Ok(self)
    }

    pub fn service_account_key(&self) -> ServiceAccountKey {
        self.service_account_key
            .clone()
            .expect("Service Account not set")
    }

    pub fn get_json_creds(&self) -> Result<String, ServiceAccountError> {
        use base64::{engine::general_purpose, Engine as _};

        Ok(std::str::from_utf8(
            &general_purpose::STANDARD.decode(self.sa_creds_base64.as_bytes())?,
        )?
        .to_string())
    }
}

fn default_gcp_location() -> String {
    "europe-west6".to_string()
}
