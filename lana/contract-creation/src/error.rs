use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ContractCreationError {
    #[error("Rendering error: {0}")]
    Rendering(#[from] rendering::RenderingError),
    #[error("Document storage error: {0}")]
    DocumentStorage(#[from] document_storage::error::DocumentStorageError),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Invalid template data: {0}")]
    InvalidTemplateData(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Job error: {0}")]
    Job(#[from] job::error::JobError),
    #[error("Authorization error: {0}")]
    Auth(#[from] authz::error::AuthorizationError),
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Document not found error")]
    NotFound,
    #[error("Customer error: {0}")]
    Customer(#[from] core_customer::error::CustomerError),
}

impl ErrorSeverity for ContractCreationError {
    fn severity(&self) -> Level {
        match self {
            Self::Rendering(e) => e.severity(),
            Self::DocumentStorage(e) => e.severity(),
            Self::TemplateNotFound(_) => Level::ERROR,
            Self::InvalidTemplateData(_) => Level::ERROR,
            Self::Io(_) => Level::ERROR,
            Self::Job(_) => Level::ERROR,
            Self::Auth(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
            Self::NotFound => Level::WARN,
            Self::Customer(e) => e.severity(),
        }
    }
}
