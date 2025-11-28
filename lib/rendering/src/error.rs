use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

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

impl ErrorSeverity for RenderingError {
    fn severity(&self) -> Level {
        match self {
            Self::Render(_) => Level::ERROR,
            Self::PdfGeneration(_) => Level::ERROR,
            Self::Io(_) => Level::ERROR,
            Self::InvalidTemplateData(_) => Level::ERROR,
        }
    }
}
