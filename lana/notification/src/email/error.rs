use handlebars::{RenderError, TemplateError};
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("EmailError - SmtpError: {0}")]
    Smtp(#[from] smtp_client::SmtpError),
    #[error("EmailError - Template: {0}")]
    Template(#[from] TemplateError),
    #[error("EmailError - Render: {0}")]
    Render(#[from] RenderError),
}

impl ErrorSeverity for EmailError {
    fn severity(&self) -> Level {
        match self {
            Self::Smtp(e) => e.severity(),
            Self::Template(_) => Level::ERROR,
            Self::Render(_) => Level::ERROR,
        }
    }
}
