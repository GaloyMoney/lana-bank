use std::collections::HashMap;
use handlebars::Handlebars;
use serde_json::json;
use uuid::Uuid;

use super::{error::LoanAgreementError, primitives::*};
use crate::primitives::CustomerId;

pub struct GenerateLoanAgreementPdf;

impl GenerateLoanAgreementPdf {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_pdf(
        &self,
        customer_id: CustomerId,
    ) -> Result<(Vec<u8>, String), LoanAgreementError> {
        let filename = format!("loan_agreement_{}.pdf", Uuid::new_v4());
        
        // Generate template data
        let template_data = self.prepare_template_data(customer_id).await?;
        
        // Render HTML template
        let html_content = self.render_html_template(&template_data)?;
        
        // Convert HTML to PDF (mock implementation)
        let pdf_data = self.html_to_pdf(&html_content).await?;
        
        Ok((pdf_data, filename))
    }

    async fn prepare_template_data(
        &self,
        customer_id: CustomerId,
    ) -> Result<HashMap<String, serde_json::Value>, LoanAgreementError> {
        let mut data = HashMap::new();
        
        // Mock customer data - in real implementation, this would fetch from customer service
        data.insert("customer_id".to_string(), json!(customer_id.to_string()));
        data.insert("customer_name".to_string(), json!("John Doe"));
        data.insert("loan_amount".to_string(), json!("$10,000.00"));
        data.insert("interest_rate".to_string(), json!("5.5%"));
        data.insert("term_months".to_string(), json!(12));
        data.insert("generation_date".to_string(), json!(chrono::Utc::now().format("%Y-%m-%d").to_string()));
        
        Ok(data)
    }

    fn render_html_template(
        &self,
        data: &HashMap<String, serde_json::Value>,
    ) -> Result<String, LoanAgreementError> {
        let mut handlebars = Handlebars::new();
        
        // Register the loan agreement template
        let template = include_str!("templates/loan_agreement.hbs");
        handlebars.register_template_string("loan_agreement", template)
            .map_err(|e| LoanAgreementError::TemplateRenderingError(e))?;
        
        let rendered = handlebars.render("loan_agreement", data)
            .map_err(|e| LoanAgreementError::TemplateRenderingError(e))?;
        
        Ok(rendered)
    }

    async fn html_to_pdf(&self, html_content: &str) -> Result<Vec<u8>, LoanAgreementError> {
        // Mock PDF generation - in real implementation, this would use a PDF generation library
        // like wkhtmltopdf, headless Chrome, or similar
        let pdf_content = format!(
            "PDF Content:\n{}\n\n--- End of PDF ---",
            html_content
        );
        
        Ok(pdf_content.into_bytes())
    }
}