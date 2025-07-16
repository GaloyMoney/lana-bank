pub mod error;
pub mod pdf;
pub mod template;

pub use error::RenderingError;
pub use pdf::PdfGenerator;
pub use template::TemplateRenderer;

/// Main rendering service that combines template processing and PDF generation
#[derive(Clone)]
pub struct Renderer {
    template_renderer: TemplateRenderer,
    pdf_generator: PdfGenerator,
}

impl Renderer {
    pub fn new(pdf_config_path: Option<std::path::PathBuf>) -> Self {
        let template_renderer = TemplateRenderer::new();
        let pdf_generator = PdfGenerator::new(pdf_config_path);

        Self {
            template_renderer,
            pdf_generator,
        }
    }

    /// Render a handlebars template and convert to PDF
    pub fn render_template_to_pdf<T: serde::Serialize>(
        &self,
        template_content: &str,
        data: &T,
    ) -> Result<Vec<u8>, RenderingError> {
        let rendered_markdown = self.template_renderer.render(template_content, data)?;
        let pdf_bytes = self
            .pdf_generator
            .generate_pdf_from_markdown(&rendered_markdown)?;
        Ok(pdf_bytes)
    }

    /// Render a handlebars template to markdown string
    pub fn render_template_to_markdown<T: serde::Serialize>(
        &self,
        template_content: &str,
        data: &T,
    ) -> Result<String, RenderingError> {
        self.template_renderer.render(template_content, data)
    }

    /// Generate PDF from markdown string
    pub fn markdown_to_pdf(&self, markdown: &str) -> Result<Vec<u8>, RenderingError> {
        self.pdf_generator.generate_pdf_from_markdown(markdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::fs;
    use std::path::Path;

    #[derive(Serialize)]
    struct TestData {
        email: String,
        name: String,
    }

    impl TestData {
        fn new(email: String) -> Self {
            Self {
                email,
                name: "Test User".to_string(),
            }
        }
    }

    #[tokio::test]
    async fn test_basic_rendering_functionality() -> Result<(), RenderingError> {
        // Get the PDF config file from the rendering library
        let pdf_config_file =
            Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("config/pdf_config.toml"));

        let test_data = TestData::new("test@example.com".to_string());

        // Test the rendering library directly
        let renderer = Renderer::new(pdf_config_file);

        // Test template content (simulate loading from file)
        let template_content = "# Test Document\n\n- **Name:** {{name}}\n- **Email:** {{email}}";

        let rendered = renderer.render_template_to_markdown(template_content, &test_data)?;

        assert!(rendered.contains("test@example.com"));
        assert!(rendered.contains("Test User"));

        // Test PDF generation - skip in CI or when fonts are not available
        if std::env::var("CI").is_ok() || std::env::var("SKIP_PDF_TESTS").is_ok() {
            // In CI environment, just test that the markdown rendering works
            // and skip the PDF generation part that requires system fonts
            eprintln!("Skipping PDF generation test in CI environment");
            return Ok(());
        }

        // Test PDF generation only in local development
        match renderer.markdown_to_pdf(&rendered) {
            Ok(pdf_bytes) => {
                assert!(!pdf_bytes.is_empty());
                assert!(pdf_bytes.starts_with(b"%PDF"));

                // Create a directory for test outputs in the rendering library
                let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-output");
                fs::create_dir_all(&output_dir)?;

                // Write the PDF to a file
                let output_path = output_dir.join("test_rendering.pdf");
                fs::write(output_path, pdf_bytes)?;
            }
            Err(e) => {
                // If PDF generation fails (e.g., due to missing fonts), log the error but don't fail the test
                eprintln!("PDF generation failed (this is expected in some environments): {}", e);
                eprintln!("To enable PDF tests locally, ensure system fonts are installed");
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_pdf_generator() -> Result<(), RenderingError> {
        let renderer = Renderer::new(None);

        let markdown = "# Test Document\n\nThis is a test.";
        let pdf_bytes = renderer.markdown_to_pdf(markdown)?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }

    #[tokio::test]
    async fn test_template_renderer() -> Result<(), RenderingError> {
        let renderer = Renderer::new(None);

        let template_content = "# Hello {{name}}\n\n- **Email:** {{email}}";
        let test_data = TestData::new("test@example.com".to_string());

        let rendered = renderer.render_template_to_markdown(template_content, &test_data)?;

        assert!(rendered.contains("test@example.com"));
        assert!(rendered.contains("Test User"));
        assert!(rendered.contains("# Hello Test User"));

        Ok(())
    }

    #[tokio::test]
    async fn test_end_to_end_template_to_pdf() -> Result<(), RenderingError> {
        let renderer = Renderer::new(None);

        let template_content = "# Loan Agreement\n\n- **Name:** {{name}}\n- **Email:** {{email}}\n\nThis is a test document.";
        let test_data = TestData::new("john.doe@example.com".to_string());

        // Test the complete flow from template to PDF
        let pdf_bytes = renderer.render_template_to_pdf(template_content, &test_data)?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }
}
