use handlebars::Handlebars;
use serde::Serialize;
use tracing_macros::record_error_severity;

use super::error::PdfGenerationError;

/// PDF template manager that handles embedded templates for various document types
#[derive(Clone)]
pub struct PdfTemplates {
    handlebars: Handlebars<'static>,
}

impl Default for PdfTemplates {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfTemplates {
    /// Create a new PDF templates instance with all embedded templates
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        // Register loan agreement template
        handlebars
            .register_template_string(
                "loan_agreement",
                include_str!("templates/loan_agreement.md.hbs"),
            )
            .expect("Could not register 'loan_agreement' template");

        // Register credit facility export template
        handlebars
            .register_template_string(
                "credit_facility_export",
                include_str!("templates/credit_facility_export.md.hbs"),
            )
            .expect("Could not register 'credit_facility_export' template");

        Self { handlebars }
    }

    /// Render a PDF template with the provided data
    #[record_error_severity]
    #[tracing::instrument(
        name = "pdf_generation.render_template",
        skip_all,
        fields(template_name = %template_name),
    )]
    pub fn render_template<T: Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<String, PdfGenerationError> {
        let rendered = self
            .handlebars
            .render(template_name, data)
            .map_err(|e| PdfGenerationError::Rendering(rendering::RenderingError::Render(e)))?;
        Ok(rendered)
    }
}
