#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
pub mod template;

use gotenberg::{GotenbergClient, GotenbergConfig};
use tracing_macros::record_error_severity;

pub use error::RenderingError;
pub use template::TemplateRenderer;

/// Main rendering service that combines template processing and PDF generation
#[derive(Clone)]
pub struct Renderer {
    template_renderer: TemplateRenderer,
    gotenberg_client: GotenbergClient,
}

impl Renderer {
    pub fn new(gotenberg_config: GotenbergConfig) -> Self {
        let template_renderer = TemplateRenderer::new();
        let gotenberg_client = GotenbergClient::new(gotenberg_config);

        Self {
            template_renderer,
            gotenberg_client,
        }
    }

    /// Render a handlebars template and convert to PDF
    #[record_error_severity]
    #[tracing::instrument(name = "rendering.render_template_to_pdf", skip_all)]
    pub async fn render_template_to_pdf(&self, content: &str) -> Result<Vec<u8>, RenderingError> {
        let pdf_bytes = self
            .gotenberg_client
            .generate_pdf_from_markdown(content)
            .await?;
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
    pub async fn markdown_to_pdf(&self, markdown: &str) -> Result<Vec<u8>, RenderingError> {
        self.gotenberg_client
            .generate_pdf_from_markdown(markdown)
            .await
            .map_err(RenderingError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    fn test_config() -> GotenbergConfig {
        GotenbergConfig::default()
    }

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
    #[ignore = "requires Gotenberg service"]
    async fn test_basic_rendering_functionality() -> Result<(), RenderingError> {
        let test_data = TestData::new("test@example.com".to_string());

        let renderer = Renderer::new(test_config());

        let template_content = "# Test Document\n\n- **Name:** {{name}}\n- **Email:** {{email}}";

        let rendered = renderer.render_template_to_markdown(template_content, &test_data)?;

        assert!(rendered.contains("test@example.com"));
        assert!(rendered.contains("Test User"));

        let pdf_bytes = renderer.markdown_to_pdf(&rendered).await?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires Gotenberg service"]
    async fn test_pdf_generator() -> Result<(), RenderingError> {
        let renderer = Renderer::new(test_config());

        let markdown = "# Test Document\n\nThis is a test.";
        let pdf_bytes = renderer.markdown_to_pdf(markdown).await?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }

    #[tokio::test]
    async fn test_template_renderer() -> Result<(), RenderingError> {
        let renderer = Renderer::new(test_config());

        let template_content = "# Hello {{name}}\n\n- **Email:** {{email}}";
        let test_data = TestData::new("test@example.com".to_string());

        let rendered = renderer.render_template_to_markdown(template_content, &test_data)?;

        assert!(rendered.contains("test@example.com"));
        assert!(rendered.contains("Test User"));
        assert!(rendered.contains("# Hello Test User"));

        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires Gotenberg service"]
    async fn test_template_to_pdf() -> Result<(), RenderingError> {
        let renderer = Renderer::new(test_config());
        let content = "# Loan Agreement\n\n- **Name:** abc@galoy.io\n";
        let pdf_bytes = renderer.render_template_to_pdf(content).await?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }
}
