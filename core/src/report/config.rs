use gcp_bigquery_client::yup_oauth2::ServiceAccountKey;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ReportConfig {
    #[serde(default)]
    pub sa_creds_base64: String,
    #[serde(skip)]
    service_account_key: Option<ServiceAccountKey>,
    #[serde(default)]
    pub gcp_project: String,
    #[serde(default)]
    pub gcp_location: String,
    #[serde(default)]
    pub dataform_repo: String,
    #[serde(default)]
    pub dataform_output_dataset: String,
    #[serde(default)]
    pub dataform_release_config: String,
    #[serde(default)]
    pub bucket_name: String,
    #[serde(default)]
    pub folder_prefix: String,
}

impl ReportConfig {
    pub fn init(
        sa_creds_base64: String,
        name_prefix: String,
        gcp_location: String,
    ) -> anyhow::Result<Self> {
        let mut cfg = Self {
            dataform_repo: format!("{}-repo", name_prefix),
            dataform_output_dataset: format!("dataform_{}", name_prefix),
            dataform_release_config: format!("{}-release", name_prefix),
            gcp_location,
            ..Default::default()
        };
        cfg.set_sa_creds_base64(sa_creds_base64)?;
        Ok(cfg)
    }

    pub fn set_sa_creds_base64(&mut self, sa_creds_base64: String) -> anyhow::Result<()> {
        use base64::{engine::general_purpose, Engine as _};

        let creds =
            std::str::from_utf8(&general_purpose::STANDARD.decode(sa_creds_base64.as_bytes())?)?
                .to_string();
        let service_account_key = serde_json::from_str::<ServiceAccountKey>(&creds)?;

        self.gcp_project = service_account_key
            .project_id
            .clone()
            .ok_or(anyhow::anyhow!("project_id missing"))?;
        self.service_account_key = Some(service_account_key);
        self.sa_creds_base64 = sa_creds_base64;

        Ok(())
    }

    pub fn service_account_key(&self) -> ServiceAccountKey {
        self.service_account_key
            .clone()
            .expect("Service Account not set")
    }
}
