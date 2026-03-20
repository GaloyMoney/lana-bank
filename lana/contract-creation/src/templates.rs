use handlebars::Handlebars;
use serde::Serialize;

use tracing_macros::record_error_severity;

use super::error::ContractCreationError;

/// Contract template manager that handles embedded templates
#[derive(Clone)]
pub struct ContractTemplates {
    handlebars: Handlebars<'static>,
}

impl Default for ContractTemplates {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractTemplates {
    /// Create a new contract templates instance with embedded templates
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string(
                "credit_facility_agreement",
                include_str!("templates/credit_facility_agreement.md.hbs"),
            )
            .expect("Could not register 'credit_facility_agreement' template");

        Self { handlebars }
    }

    /// Render a contract template with the provided data
    #[record_error_severity]
    #[tracing::instrument(
        name = "lana.contract_creation.render_template",
        skip_all,
        fields(template_name = %template_name),
    )]
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
    pub fn get_credit_facility_agreement_template(
        &self,
        data: &impl Serialize,
    ) -> Result<String, ContractCreationError> {
        self.render_template("credit_facility_agreement", data)
    }
}
