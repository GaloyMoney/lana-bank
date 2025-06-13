use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplatingConfig {
    pub template_dir: PathBuf,
    pub pdf: PdfConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PdfConfig {
    pub temp_dir: Option<PathBuf>,
    pub cleanup_temp_files: bool,
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
            temp_dir: None,
            cleanup_temp_files: true,
        }
    }
}
