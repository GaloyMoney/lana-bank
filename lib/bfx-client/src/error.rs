use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum BfxClientError {
    #[error("BfxClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("BfxClientError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("BfxClientError - ConversionError: {0}")]
    ConversionError(#[from] money::ConversionError),
    #[error("BfxClientError - UnexpectedResponse: {event:?} {code:?} - {desc:?}")]
    UnexpectedResponse {
        event: String,
        code: u32,
        desc: String,
    },
    #[error("BfxClientError - RequestParametersError: {event:?} {code:?} - {desc:?}")]
    RequestParametersError {
        event: String,
        code: u32,
        desc: String,
    },
}

impl From<(String, u32, String)> for BfxClientError {
    fn from((event, code, desc): (String, u32, String)) -> Self {
        match code {
            10020 => BfxClientError::RequestParametersError { event, code, desc },
            _ => BfxClientError::UnexpectedResponse { event, code, desc },
        }
    }
}

impl ErrorSeverity for BfxClientError {
    fn severity(&self) -> Level {
        match self {
            Self::Reqwest(_) => Level::WARN,
            Self::Deserialization(_) => Level::ERROR,
            Self::ConversionError(e) => e.severity(),
            Self::UnexpectedResponse { .. } => Level::ERROR,
            Self::RequestParametersError { .. } => Level::ERROR,
        }
    }
}
