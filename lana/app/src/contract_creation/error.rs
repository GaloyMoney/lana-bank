use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractCreationError {
    #[error("Rendering error: {0}")]
    Rendering(#[from] rendering::RenderingError),
    #[error("Customer error: {0}")]
    Customer(#[from] crate::customer::error::CustomerError),
    #[error("Document storage error: {0}")]
    DocumentStorage(#[from] document_storage::error::DocumentStorageError),
    #[error("Authorization error: {0}")]
    Authorization(#[from] crate::authorization::error::AuthorizationError),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Invalid template data: {0}")]
    InvalidTemplateData(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Job error: {0}")]
    Job(#[from] job::error::JobError),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Document not found error")]
    NotFound,
}
