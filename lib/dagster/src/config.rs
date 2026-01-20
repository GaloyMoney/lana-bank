use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DagsterConfig {
    #[serde(default = "default_uri")]
    pub uri: Url,
    #[serde(default = "default_pipeline_name_for_report_generation")]
    pub pipeline_name_for_report_generation: String,
    #[serde(default = "default_repository_location_name")]
    pub repository_location_name: String,
    #[serde(default = "default_repository_name")]
    pub repository_name: String,
}

impl Default for DagsterConfig {
    fn default() -> Self {
        Self {
            uri: default_uri(),
            pipeline_name_for_report_generation: default_pipeline_name_for_report_generation(),
            repository_location_name: default_repository_location_name(),
            repository_name: default_repository_name(),
        }
    }
}

fn default_uri() -> Url {
    Url::parse("http://localhost:3000/graphql").expect("invalid url")
}

fn default_pipeline_name_for_report_generation() -> String {
    "file_reports_generation".to_string()
}

fn default_repository_location_name() -> String {
    "Lana DW".to_string()
}

fn default_repository_name() -> String {
    "__repository__".to_string()
}
