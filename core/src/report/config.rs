#[derive(Clone)]
pub struct ReportConfig {
    pub sa_creds_base64: String,
    pub gcp_project: String,
    pub gcp_location: String,
    pub dataform_repo: String,
    pub dataform_release_config: String,
}
