use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DagsterError {
    #[error("DagsterError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("DagsterError - UrlParse: {0}")]
    Url(#[from] url::ParseError),
    #[error("DagsterError - ApiError")]
    ApiError,
}

impl ErrorSeverity for DagsterError {
    fn severity(&self) -> Level {
        match self {
            Self::Reqwest(_) => Level::ERROR,
            Self::Url(_) => Level::ERROR,
            Self::ApiError => Level::ERROR,
        }
    }
}
