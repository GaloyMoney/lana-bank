use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractCreationError {
    #[error("Rendering error: {0}")]
    Rendering(#[from] rendering::RenderingError),
    #[error("Customer error: {0}")]
    Customer(#[from] crate::customer::error::CustomerError),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Invalid template data: {0}")]
    InvalidTemplateData(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
