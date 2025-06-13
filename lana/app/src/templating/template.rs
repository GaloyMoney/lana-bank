use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

use crate::templating::error::TemplatingError;

#[derive(Clone)]
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    template_dir: PathBuf,
}

impl TemplateEngine {
    pub async fn init(template_dir: PathBuf) -> Result<Self, TemplatingError> {
        let handlebars = Handlebars::new();

        // Load all templates from the template directory
        let mut engine = Self {
            handlebars,
            template_dir,
        };

        engine.load_templates().await?;

        Ok(engine)
    }

    async fn load_templates(&mut self) -> Result<(), TemplatingError> {
        if !self.template_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.template_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "hbs" {
                        let template_name =
                            path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                                TemplatingError::InvalidTemplateData(format!(
                                    "Invalid template filename: {:?}",
                                    path
                                ))
                            })?;

                        // Handle double extensions like .md.hbs - get just the base name
                        let template_name =
                            if let Some(stripped) = template_name.strip_suffix(".md") {
                                stripped
                            } else {
                                template_name
                            };

                        let template_content = fs::read_to_string(&path)?;
                        self.handlebars
                            .register_template_string(template_name, &template_content)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn render<T: Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<String, TemplatingError> {
        if !self.handlebars.has_template(template_name) {
            return Err(TemplatingError::TemplateNotFound(template_name.to_string()));
        }

        let rendered = self.handlebars.render(template_name, data)?;
        Ok(rendered)
    }

    pub fn list_templates(&self) -> Vec<String> {
        self.handlebars.get_templates().keys().cloned().collect()
    }
}

/// Data structure for loan agreement template
#[derive(Serialize)]
pub struct LoanAgreementData {
    pub email: String,
}

impl LoanAgreementData {
    pub fn new(email: String) -> Self {
        Self { email }
    }
}
