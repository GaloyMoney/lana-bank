use cloud_storage::Storage;
use dagster::DagsterConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ReportConfig {
    #[serde(default)]
    pub dagster: DagsterConfig,
    #[serde(default)]
    pub reports_bucket_name: Option<String>,
}

impl ReportConfig {
    /// Build the storage backend for report files.
    ///
    /// Uses a dedicated GCP bucket when `reports_bucket_name` is set,
    /// otherwise falls back to the provided default storage.
    pub fn report_file_storage(&self, default: &Storage) -> Storage {
        self.reports_bucket_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|bucket_name| {
                Storage::new(&cloud_storage::config::StorageConfig::new_gcp(
                    bucket_name.to_owned(),
                ))
            })
            .unwrap_or_else(|| default.clone())
    }
}
