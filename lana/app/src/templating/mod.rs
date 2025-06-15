// Re-export the main templating library
pub use lib::*;

pub mod config;
pub mod error;
mod lib;
mod pdf;
mod template;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[tokio::test]
    async fn test_basic_templating_functionality() -> Result<(), error::TemplatingError> {
        // Create a custom config with absolute paths
        let config_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templating/config/pdf_config.toml");
        let template_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templating/templates");

        let mut config = config::TemplatingConfig::default();
        config.pdf.config_file = config_file;
        config.template_dir = template_dir;

        // Note: This test focuses on testing the core templating functionality
        // without requiring customer dependencies. For full integration testing
        // with customers, use the actual application context.

        let loan_data = template::LoanAgreementData::new("test@example.com".to_string());

        // Test template engine independently
        let template_engine = template::TemplateEngine::init(config.template_dir.clone()).await?;
        let rendered = template_engine.render("loan_agreement", &loan_data).await?;

        assert!(rendered.contains("test@example.com"));

        // Test PDF generator independently
        let pdf_generator = pdf::PdfGenerator::init(config.pdf).await?;
        let pdf_bytes = pdf_generator.generate_pdf_from_markdown(&rendered).await?;

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
        let pdf_bytes = pdf_generator.generate_pdf_from_markdown(markdown).await?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }

    #[tokio::test]
    async fn test_template_engine() -> Result<(), error::TemplatingError> {
        let template_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templating/templates");
        let template_engine = template::TemplateEngine::init(template_dir).await?;

        let loan_data = template::LoanAgreementData::new("test@example.com".to_string());

        let rendered = template_engine.render("loan_agreement", &loan_data).await?;

        assert!(rendered.contains("test@example.com"));

        Ok(())
    }
}
