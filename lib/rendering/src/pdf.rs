use reqwest::multipart::{Form, Part};

use crate::error::RenderingError;

// HTML wrapper template for Gotenberg's Markdown route
const PDF_WRAPPER_HTML: &str = include_str!("../config/pdf_wrapper.html");

/// PDF generator that converts markdown to PDF via Gotenberg
#[derive(Clone)]
pub struct PdfGenerator {
    client: reqwest::Client,
    gotenberg_url: String,
}

impl PdfGenerator {
    /// Create a new PDF generator with Gotenberg URL
    pub fn new(gotenberg_url: String) -> Self {
        let client = reqwest::Client::builder()
            .default_headers(tracing_utils::http::inject_trace_reqwest())
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            gotenberg_url,
        }
    }

    /// Generate a PDF from markdown content via Gotenberg
    /// Returns the PDF as bytes that can be written to a file or uploaded
    pub async fn generate_pdf_from_markdown(
        &self,
        markdown: &str,
    ) -> Result<Vec<u8>, RenderingError> {
        let form = Form::new()
            .part(
                "files",
                Part::text(PDF_WRAPPER_HTML.to_string())
                    .file_name("index.html")
                    .mime_str("text/html")
                    .map_err(|e| RenderingError::PdfGeneration(e.to_string()))?,
            )
            .part(
                "files",
                Part::text(markdown.to_string())
                    .file_name("content.md")
                    .mime_str("text/markdown")
                    .map_err(|e| RenderingError::PdfGeneration(e.to_string()))?,
            );

        let url = format!("{}/forms/chromium/convert/markdown", self.gotenberg_url);
        let response = self.client.post(&url).multipart(form).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            return Err(RenderingError::Gotenberg(format!(
                "Gotenberg returned status {status}: {body}"
            )));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
