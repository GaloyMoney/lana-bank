use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderingError {
    #[error("Render error: {0}")]
    Render(#[from] handlebars::RenderError),
    #[error("PDF generation error: {0}")]
    PdfGeneration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid template data: {0}")]
    InvalidTemplateData(String),
}
