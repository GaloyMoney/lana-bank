use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum AirflowError {
    #[error("AirflowError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("AirflowError - UrlParse: {0}")]
    Url(#[from] url::ParseError),
    #[error("AirflowError - ApiError")]
    ApiError,
}

impl ErrorSeverity for AirflowError {
    fn severity(&self) -> Level {
        match self {
            Self::Reqwest(_) => Level::ERROR,
            Self::Url(_) => Level::ERROR,
            Self::ApiError => Level::ERROR,
        }
    }
}
