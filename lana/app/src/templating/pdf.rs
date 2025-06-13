use std::fs;

use uuid::Uuid;

use crate::templating::config::PdfConfig;
use crate::templating::error::TemplatingError;

pub struct PdfGenerator {
    config: PdfConfig,
}

impl PdfGenerator {
    pub async fn init(config: PdfConfig) -> Result<Self, TemplatingError> {
        Ok(Self { config })
    }

    pub async fn from_markdown(&self, markdown: &str) -> Result<Vec<u8>, TemplatingError> {
        let temp_dir = self
            .config
            .temp_dir
            .clone()
            .unwrap_or_else(std::env::temp_dir);
        let temp_file_name = temp_dir.join(format!("{}.pdf", Uuid::new_v4()));

        markdown2pdf::parse(markdown.to_string(), &temp_file_name.to_string_lossy())
            .map_err(|e| TemplatingError::PdfGeneration(format!("PDF generation failed: {}", e)))?;
        let pdf_bytes = fs::read(&temp_file_name)?;

        if self.config.cleanup_temp_files {
            let _ = fs::remove_file(&temp_file_name); // Ignore cleanup errors
        }

        Ok(pdf_bytes)
    }

    pub async fn from_html(&self, html: &str) -> Result<Vec<u8>, TemplatingError> {
        // Convert HTML to markdown first, then to PDF
        // This is a simplified approach - you might want to use a proper HTML to PDF converter
        // For now, just treat HTML as markdown (basic support)
        self.from_markdown(html).await
    }
}
