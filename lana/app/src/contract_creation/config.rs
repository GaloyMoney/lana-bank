use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContractCreationConfig {
    pub template_dir: PathBuf,
    pub pdf_config_file: Option<PathBuf>,
}

impl Default for ContractCreationConfig {
    fn default() -> Self {
        Self {
            template_dir: PathBuf::from("lana/app/src/contract_creation/templates"),
            pdf_config_file: Some(PathBuf::from("lib/rendering/config/pdf_config.toml")),
        }
    }
}
