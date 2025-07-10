use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::RenderingError;

/// PDF generator that converts markdown to PDF files
#[derive(Clone)]
pub struct PdfGenerator {
    config_file: Option<PathBuf>,
}

impl PdfGenerator {
    /// Create a new PDF generator with optional config file path
    pub fn new(config_file: Option<PathBuf>) -> Result<Self, RenderingError> {
        if let Some(ref config_path) = config_file {
            if !config_path.exists() {
                return Err(RenderingError::InvalidTemplateData(format!(
                    "PDF config file not found: {}",
                    config_path.display()
                )));
            }
        }

        Ok(Self { config_file })
    }

    /// Generate a PDF from markdown content
    /// Returns the PDF as bytes that can be written to a file or uploaded
    pub fn generate_pdf_from_markdown(&self, markdown: &str) -> Result<Vec<u8>, RenderingError> {
        let temp_dir = std::env::temp_dir();
        let temp_file_name = temp_dir.join(format!("{}.pdf", Uuid::new_v4()));

        // Convert config file path to string for the API
        let config_path_string = self
            .config_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let config_path = config_path_string.as_deref();

        // Generate the PDF using the new API that accepts config path directly
        let result = markdown2pdf::parse(
            markdown.to_string(),
            &temp_file_name.to_string_lossy(),
            config_path,
        )
        .map_err(|e| RenderingError::PdfGeneration(format!("PDF generation failed: {e}")));

        // Check if PDF generation was successful
        result?;

        let pdf_bytes = fs::read(&temp_file_name)?;

        // Clean up the temporary PDF file
        let _ = fs::remove_file(&temp_file_name);

        Ok(pdf_bytes)
    }
}
