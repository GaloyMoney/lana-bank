use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::RenderingError;

/// PDF generator that converts markdown to PDF files
#[derive(Clone)]
pub struct PdfGenerator {
    config_file: Option<PathBuf>,
}

impl PdfGenerator {
    /// Create a new PDF generator with optional config file path
    pub async fn new(config_file: Option<PathBuf>) -> Result<Self, RenderingError> {
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

        // TODO: HACK - This is a workaround for markdown2pdf's limited API
        // The markdown2pdf crate only supports configuration via a file named
        // "markdown2pdfrc.toml" in the home directory. It doesn't provide an API
        // to pass configuration directly or specify a custom config file path.
        //
        // Better solutions for the future:
        // 1. Fork markdown2pdf and add an API that accepts config directly
        // 2. Switch to a different PDF generation library with better API
        // 3. Contribute to markdown2pdf to add config parameter support
        //
        // For now, we temporarily copy our config to ~/markdown2pdfrc.toml,
        // generate the PDF, then clean up. This is not ideal because:
        // - It modifies the user's home directory
        // - It could interfere with other markdown2pdf usage
        // - It's not thread-safe if multiple PDFs are generated concurrently

        let backup_file = if let Some(ref config_file) = self.config_file {
            // Get the home directory and copy our config file there temporarily
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| {
                    RenderingError::PdfGeneration("Failed to get home directory".to_string())
                })?;

            let home_config_file = Path::new(&home_dir).join("markdown2pdfrc.toml");

            // Check if a config file already exists in home directory
            let backup_needed = home_config_file.exists();
            let backup_file = if backup_needed {
                let backup_path = Path::new(&home_dir).join("markdown2pdfrc.toml.backup");
                fs::copy(&home_config_file, &backup_path).map_err(|e| {
                    RenderingError::PdfGeneration(format!(
                        "Failed to backup existing config: {}",
                        e
                    ))
                })?;
                Some(backup_path)
            } else {
                None
            };

            // Copy our config file to the home directory
            fs::copy(config_file, &home_config_file).map_err(|e| {
                RenderingError::PdfGeneration(format!("Failed to copy config file to home: {}", e))
            })?;

            backup_file
        } else {
            None
        };

        // Generate the PDF
        let result = markdown2pdf::parse(markdown.to_string(), &temp_file_name.to_string_lossy())
            .map_err(|e| RenderingError::PdfGeneration(format!("PDF generation failed: {}", e)));

        // Clean up: remove our config file and restore backup if needed
        if self.config_file.is_some() {
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .ok();
            if let Some(home_dir) = home_dir {
                let home_config_file = Path::new(&home_dir).join("markdown2pdfrc.toml");
                let _ = fs::remove_file(&home_config_file);
                if let Some(backup_path) = backup_file {
                    let _ = fs::copy(&backup_path, &home_config_file);
                    let _ = fs::remove_file(&backup_path);
                }
            }
        }

        // Check if PDF generation was successful
        result?;

        let pdf_bytes = fs::read(&temp_file_name)?;

        // Clean up the temporary PDF file
        let _ = fs::remove_file(&temp_file_name);

        Ok(pdf_bytes)
    }
}
