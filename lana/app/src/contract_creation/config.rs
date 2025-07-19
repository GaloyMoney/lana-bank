use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContractCreationConfig {
    pub pdf_config_file: Option<PathBuf>,
}

impl Default for ContractCreationConfig {
    fn default() -> Self {
        Self {
            pdf_config_file: Some(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../lib/rendering/config/pdf_config.toml"),
            ),
        }
    }
}
