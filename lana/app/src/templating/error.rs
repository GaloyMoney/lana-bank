use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplatingError {
    #[error("Template error: {0}")]
    Template(#[from] handlebars::TemplateError),

    #[error("Render error: {0}")]
    Render(#[from] handlebars::RenderError),

    #[error("PDF generation error: {0}")]
    PdfGeneration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Customer error: {0}")]
    Customer(#[from] crate::customer::error::CustomerError),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid template data: {0}")]
    InvalidTemplateData(String),
}
