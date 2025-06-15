use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::templating::config::PdfConfig;
use crate::templating::error::TemplatingError;

#[derive(Clone)]
pub struct PdfGenerator {
    config: PdfConfig,
}

impl PdfGenerator {
    pub async fn init(config: PdfConfig) -> Result<Self, TemplatingError> {
        // Try to find the config file in several locations
        let config_file = if config.config_file.exists() {
            config.config_file.clone()
        } else {
            // Try relative to current working directory
            let cwd_path = std::env::current_dir()
                .map_err(|e| {
                    TemplatingError::InvalidTemplateData(format!(
                        "Failed to get current directory: {}",
                        e
                    ))
                })?
                .join(&config.config_file);

            if cwd_path.exists() {
                cwd_path
            } else {
                // Try finding relative to the app directory
                let app_path = Path::new("app/src/templating/config/pdf_config.toml");
                if app_path.exists() {
                    app_path.to_path_buf()
                } else {
                    // Last resort: try ../app/src/templating/config/pdf_config.toml
                    let up_one_path = Path::new("../app/src/templating/config/pdf_config.toml");
                    if up_one_path.exists() {
                        up_one_path.to_path_buf()
                    } else {
                        return Err(TemplatingError::InvalidTemplateData(format!(
                            "PDF config file not found. Tried:\n  - {}\n  - {}\n  - {}\n  - {}",
                            config.config_file.display(),
                            cwd_path.display(),
                            app_path.display(),
                            up_one_path.display()
                        )));
                    }
                }
            }
        };

        let config = PdfConfig { config_file };

        Ok(Self { config })
    }

    pub async fn generate_pdf_from_markdown(
        &self,
        markdown: &str,
    ) -> Result<Vec<u8>, TemplatingError> {
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

        // Get the home directory and copy our config file there temporarily
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                TemplatingError::PdfGeneration("Failed to get home directory".to_string())
            })?;

        let home_config_file = Path::new(&home_dir).join("markdown2pdfrc.toml");

        // Check if a config file already exists in home directory
        let backup_needed = home_config_file.exists();
        let backup_file = if backup_needed {
            let backup_path = Path::new(&home_dir).join("markdown2pdfrc.toml.backup");
            fs::copy(&home_config_file, &backup_path).map_err(|e| {
                TemplatingError::PdfGeneration(format!("Failed to backup existing config: {}", e))
            })?;
            Some(backup_path)
        } else {
            None
        };

        // Copy our config file to the home directory
        fs::copy(&self.config.config_file, &home_config_file).map_err(|e| {
            TemplatingError::PdfGeneration(format!("Failed to copy config file to home: {}", e))
        })?;

        // Generate the PDF
        let result = markdown2pdf::parse(markdown.to_string(), &temp_file_name.to_string_lossy())
            .map_err(|e| TemplatingError::PdfGeneration(format!("PDF generation failed: {}", e)));

        // Clean up: remove our config file and restore backup if needed
        let _ = fs::remove_file(&home_config_file);
        if let Some(backup_path) = backup_file {
            let _ = fs::copy(&backup_path, &home_config_file);
            let _ = fs::remove_file(&backup_path);
        }

        // Check if PDF generation was successful
        result?;

        let pdf_bytes = fs::read(&temp_file_name)?;

        // Clean up the temporary PDF file
        let _ = fs::remove_file(&temp_file_name);

        Ok(pdf_bytes)
    }
}
