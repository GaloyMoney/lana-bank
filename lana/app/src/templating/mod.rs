use anyhow::Result;
use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use uuid::Uuid;

use crate::customer::{CustomerId, Customers};

/// Converts markdown content to PDF bytes
pub fn from_markdown(markdown: &str) -> Result<Vec<u8>> {
    let temp_dir = std::env::temp_dir();
    let temp_file_name = temp_dir.join(format!("{}.pdf", Uuid::new_v4()));
    markdown2pdf::parse(markdown.to_string(), &temp_file_name.to_string_lossy())?;
    let pdf_bytes = fs::read(&temp_file_name)?;
    fs::remove_file(&temp_file_name)?;
    Ok(pdf_bytes)
}

/// Creates a PDF from a template file and data structure
pub fn create_pdf_from_template<T: Serialize>(template_path: &str, data: &T) -> Result<Vec<u8>> {
    let markdown_template = fs::read_to_string(template_path)?;
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("template", &markdown_template)?;
    let rendered_markdown = handlebars.render("template", data)?;
    from_markdown(&rendered_markdown)
}

/// Data structure for loan agreement template
#[derive(Serialize)]
pub struct LoanAgreementData {
    pub name: String,
    pub email: String,
    pub amount: String,
}

impl LoanAgreementData {
    pub fn new(name: String, email: String, amount: String) -> Self {
        Self {
            name,
            email,
            amount,
        }
    }
}

/// Creates a loan agreement PDF by fetching customer data from the database
///
/// # Example
/// ```ignore
/// // In your service layer or use case:
/// let customers = app.customer(); // Get customer repository
/// let customer_id = CustomerId::new(); // Your customer ID
/// let pdf_bytes = create_loan_agreement_pdf_for_customer(
///     &customers,
///     customer_id,
///     "John Doe".to_string(),
///     "$10,000".to_string(),
/// ).await?;
/// ```
pub async fn create_loan_agreement_pdf_for_customer(
    customers: &Customers,
    customer_id: CustomerId,
    name: String,
    amount: String,
) -> Result<Vec<u8>> {
    // Fetch customer data from database
    let customer = customers.find_by_id_without_audit(customer_id).await?;

    // Create loan agreement data with customer email
    let loan_data = LoanAgreementData::new(name, customer.email.clone(), amount);

    // Generate PDF from template
    create_pdf_from_template("src/templating/templates/loan_agreement.md", &loan_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_create_pdf_from_template() -> Result<()> {
        let loan_data = LoanAgreementData::new(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "$1000".to_string(),
        );

        let pdf_bytes =
            create_pdf_from_template("src/templating/templates/loan_agreement.md", &loan_data)?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        // Create a directory for test outputs
        let output_dir = Path::new("test-output");
        fs::create_dir_all(output_dir)?;

        // Write the PDF to a file
        let output_path = output_dir.join("test_loan_agreement.pdf");
        fs::write(output_path, pdf_bytes)?;

        Ok(())
    }

    // Note: Testing create_loan_agreement_pdf_for_customer would require
    // setting up database connections and customer repository, which is
    // better suited for integration tests
}
