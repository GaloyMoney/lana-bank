use std::path::PathBuf;

use crate::error::RenderingError;

/// PDF generator that converts markdown to PDF files
#[derive(Clone)]
pub struct PdfGenerator {
    config_file: Option<PathBuf>,
}

impl PdfGenerator {
    /// Create a new PDF generator with optional config file path
    pub fn new(config_file: Option<PathBuf>) -> Self {
        if let Some(ref config_path) = config_file {
            assert!(
                config_path.exists(),
                "PDF config file not found: {}",
                config_path.display()
            );
        }

        Self { config_file }
    }

    /// Generate a PDF from markdown content
    /// Returns the PDF as bytes that can be written to a file or uploaded
    pub fn generate_pdf_from_markdown(&self, markdown: &str) -> Result<Vec<u8>, RenderingError> {
        // Convert config file path to string for the API
        let config_path_string = self
            .config_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let config_path = config_path_string.as_deref();

        // Generate the PDF directly into bytes without temporary file
        markdown2pdf::parse_into_bytes(markdown.to_string(), config_path)
            .map_err(|e| RenderingError::PdfGeneration(format!("PDF generation failed: {e}")))
    }
}
