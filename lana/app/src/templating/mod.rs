// Re-export the main templating library
pub use lib::*;

mod config;
mod error;
mod lib;
mod pdf;
mod template;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[tokio::test]
    async fn test_templating_library() -> Result<(), error::TemplatingError> {
        let config = config::TemplatingConfig::default();
        let templating = lib::Templating::init(config).await?;

        let loan_data = template::LoanAgreementData::new(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "$1000".to_string(),
        );

        let pdf_bytes = templating
            .generate_pdf_from_template("loan_agreement", &loan_data)
            .await?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        // Create a directory for test outputs
        let output_dir = Path::new("test-output");
        fs::create_dir_all(output_dir).map_err(|e| error::TemplatingError::Io(e))?;

        // Write the PDF to a file
        let output_path = output_dir.join("test_loan_agreement.pdf");
        fs::write(output_path, pdf_bytes).map_err(|e| error::TemplatingError::Io(e))?;

        Ok(())
    }

    #[tokio::test]
    async fn test_pdf_generator() -> Result<(), error::TemplatingError> {
        let config = config::PdfConfig::default();
        let pdf_generator = pdf::PdfGenerator::init(config).await?;

        let markdown = "# Test Document\n\nThis is a test.";
        let pdf_bytes = pdf_generator.from_markdown(markdown).await?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }

    #[tokio::test]
    async fn test_template_engine() -> Result<(), error::TemplatingError> {
        use std::path::PathBuf;

        let template_dir = PathBuf::from("src/templating/templates");
        let template_engine = template::TemplateEngine::init(template_dir).await?;

        let loan_data = template::LoanAgreementData::new(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "$1000".to_string(),
        );

        let rendered = template_engine.render("loan_agreement", &loan_data).await?;

        assert!(rendered.contains("Test User"));
        assert!(rendered.contains("test@example.com"));
        assert!(rendered.contains("$1000"));

        Ok(())
    }
}
