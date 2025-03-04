use std::path::{Path, PathBuf};

use crate::email::error::EmailError;

#[derive(Clone)]
pub struct EmailTemplate {
    templates_path: PathBuf,
}

impl EmailTemplate {
    pub fn new<P: AsRef<Path>>(templates_path: P) -> Result<Self, EmailError> {
        unimplemented!()
    }

    pub fn render(
        &self,
        template_name: &str,
        context: &serde_json::Value,
    ) -> Result<String, EmailError> {
        unimplemented!()
    }
}
