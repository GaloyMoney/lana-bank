use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("ExportError - CalaError: {0}")]
    Cala(#[from] super::cala::error::CalaError),
}
