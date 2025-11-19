use crate::error::RenderingError;

// Default PDF configuration embedded in the binary
const DEFAULT_PDF_CONFIG: &str = include_str!("../config/pdf_config.toml");

/// PDF generator that converts markdown to PDF files
#[derive(Clone)]
pub struct PdfGenerator;

impl PdfGenerator {
    /// Create a new PDF generator with embedded config
    pub fn new() -> Self {
        Self
    }

    /// Generate a PDF from markdown content
    /// Returns the PDF as bytes that can be written to a file or uploaded
    pub fn generate_pdf_from_markdown(&self, markdown: &str) -> Result<Vec<u8>, RenderingError> {
        use markdown2pdf::config::ConfigSource;

        // Use the embedded config directly
        let config_source = ConfigSource::Embedded(DEFAULT_PDF_CONFIG);

        markdown2pdf::parse_into_bytes(markdown.to_string(), config_source, None)
            .map_err(|e| RenderingError::PdfGeneration(format!("PDF generation failed: {e}")))
    }
}

impl Default for PdfGenerator {
    fn default() -> Self {
        Self::new()
    }
}
