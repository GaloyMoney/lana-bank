use handlebars::Handlebars;
use serde::Serialize;

use super::error::ContractCreationError;

/// Contract template manager that handles embedded templates
#[derive(Clone)]
pub struct ContractTemplates {
    handlebars: Handlebars<'static>,
}

impl ContractTemplates {
    /// Create a new contract templates instance with embedded templates
    pub fn init() -> Result<Self, ContractCreationError> {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string(
                "loan_agreement",
                include_str!("templates/loan_agreement.md.hbs"),
            )
            .map_err(|e| {
                ContractCreationError::Rendering(rendering::RenderingError::Template(e))
            })?;

        Ok(Self { handlebars })
    }

    /// Render a contract template with the provided data
    pub fn render_template<T: Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<String, ContractCreationError> {
        let rendered = self
            .handlebars
            .render(template_name, data)
            .map_err(|e| ContractCreationError::Rendering(rendering::RenderingError::Render(e)))?;
        Ok(rendered)
    }

    /// Get the loan agreement template content
    pub fn get_loan_agreement_template(
        &self,
        data: &impl Serialize,
    ) -> Result<String, ContractCreationError> {
        self.render_template("loan_agreement", data)
    }
}

impl Default for ContractTemplates {
    fn default() -> Self {
        Self::init().expect("Failed to initialize contract templates")
    }
}
