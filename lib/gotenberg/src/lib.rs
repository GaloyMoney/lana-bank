#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

use reqwest::Url;
use reqwest::multipart::{Form, Part};
use tracing_macros::record_error_severity;

pub use config::GotenbergConfig;
pub use error::GotenbergError;

// HTML wrapper template for Gotenberg's Markdown route
const PDF_WRAPPER_HTML: &str = include_str!("../config/pdf_wrapper.html");

/// Gotenberg client for PDF generation
#[derive(Clone)]
pub struct GotenbergClient {
    client: reqwest::Client,
    url: Url,
}

impl GotenbergClient {
    /// Create a new Gotenberg client
    pub fn new(config: GotenbergConfig) -> Self {
        let client = reqwest::Client::builder()
            .default_headers(tracing_utils::http::inject_trace_reqwest())
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            url: config.url,
        }
    }

    /// Generate a PDF from markdown content via Gotenberg
    /// Returns the PDF as bytes that can be written to a file or uploaded
    #[record_error_severity]
    #[tracing::instrument(name = "gotenberg.generate_pdf_from_markdown", skip_all)]
    pub async fn generate_pdf_from_markdown(
        &self,
        markdown: &str,
    ) -> Result<Vec<u8>, GotenbergError> {
        let form = Form::new()
            .part(
                "files",
                Part::text(PDF_WRAPPER_HTML.to_string())
                    .file_name("index.html")
                    .mime_str("text/html")
                    .map_err(|e| GotenbergError::Multipart(e.to_string()))?,
            )
            .part(
                "files",
                Part::text(markdown.to_string())
                    .file_name("content.md")
                    .mime_str("text/markdown")
                    .map_err(|e| GotenbergError::Multipart(e.to_string()))?,
            );

        let url = self
            .url
            .join("forms/chromium/convert/markdown")
            .expect("valid URL path");
        let response = self.client.post(url).multipart(form).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            return Err(GotenbergError::Server(format!(
                "Gotenberg returned status {status}: {body}"
            )));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> GotenbergConfig {
        GotenbergConfig::default()
    }

    #[tokio::test]
    #[ignore = "requires Gotenberg service"]
    async fn test_generate_pdf_from_markdown() -> Result<(), GotenbergError> {
        let client = GotenbergClient::new(test_config());

        let markdown = "# Test Document\n\nThis is a test.";
        let pdf_bytes = client.generate_pdf_from_markdown(markdown).await?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        Ok(())
    }
}
