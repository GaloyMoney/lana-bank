use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DagsterError {
    #[error("DagsterError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("DagsterError - SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("DagsterError - PythonError: {0}")]
    PythonError(String),
    #[error("DagsterError - ApiError")]
    ApiError,
}

impl ErrorSeverity for DagsterError {
    fn severity(&self) -> Level {
        match self {
            Self::Reqwest(_) => Level::ERROR,
            Self::SerdeJson(_) => Level::ERROR,
            Self::PythonError(_) => Level::ERROR,
            Self::ApiError => Level::ERROR,
        }
    }
}
