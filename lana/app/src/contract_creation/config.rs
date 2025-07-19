use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContractCreationConfig {
    pub pdf_config_file: Option<PathBuf>,
}

impl Default for ContractCreationConfig {
    fn default() -> Self {
        Self {
            // pdf_config_file: Some(PathBuf::from("lib/rendering/config/pdf_config.toml"))

            // return None for now
            // revisit when https://github.com/theiskaa/markdown2pdf/issues/41 is fixed
            pdf_config_file: None,
        }
    }
}
