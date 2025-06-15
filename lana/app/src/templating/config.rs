use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplatingConfig {
    pub template_dir: PathBuf,
    pub pdf: PdfConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PdfConfig {
    /// Path to the TOML configuration file for PDF rendering
    pub config_file: PathBuf,
}

impl Default for TemplatingConfig {
    fn default() -> Self {
        Self {
            template_dir: PathBuf::from("src/templating/templates"),
            pdf: PdfConfig::default(),
        }
    }
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            config_file: PathBuf::from("app/src/templating/config/pdf_config.toml"),
        }
    }
}
