// Re-export the main contract creation library
pub use lib::*;

pub mod config;
pub mod error;
pub mod job;
mod lib;

pub use job::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_contract_creation_config() -> Result<(), error::ContractCreationError> {
        // Test that config works correctly
        let template_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/contract_creation/templates");
        let pdf_config_file = Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../lib/rendering/config/pdf_config.toml"),
        );

        let mut config = config::ContractCreationConfig::default();
        config.template_dir = template_dir;
        config.pdf_config_file = pdf_config_file;

        // Verify that config can be used to create a renderer
        let renderer = rendering::Renderer::new(config.pdf_config_file)
            .await
            .map_err(|e| error::ContractCreationError::Rendering(e))?;

        // Test basic functionality
        let template_content = "# Test Contract\n\nHello {{name}}!";
        let data = serde_json::json!({"name": "World"});

        let result = renderer
            .render_template_to_markdown(template_content, &data)
            .map_err(|e| error::ContractCreationError::Rendering(e))?;

        assert!(result.contains("Hello World!"));

        Ok(())
    }
}
