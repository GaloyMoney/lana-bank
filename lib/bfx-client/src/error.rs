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
    #[error("BfxClientError - RequestParametersError: {code} - {desc}")]
    RequestParametersError { code: u32, desc: String },
    #[error("BfxClientError - InvalidCredentials: {code} - {desc}")]
    InvalidCredentials { code: u32, desc: String },
    #[error("BfxClientError - UnexpectedResponse: {event:?} {code:?} - {desc}")]
    UnexpectedResponse {
        event: Option<String>,
        code: Option<u32>,
        desc: String,
    },
    #[error("BfxClientError - UnexpectedHttpStatus: {status} - {body}")]
    UnexpectedHttpStatus {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("BfxClientError - InvalidNotificationStatus: {status} - {text}")]
    InvalidNotificationStatus { status: String, text: String },
}

impl From<(String, u32, String)> for BfxClientError {
    fn from((event, code, desc): (String, u32, String)) -> Self {
        match code {
            10020 => BfxClientError::RequestParametersError { code, desc },
            10100 => BfxClientError::InvalidCredentials { code, desc },
            _ => BfxClientError::UnexpectedResponse {
                event: Some(event),
                code: Some(code),
                desc,
            },
        }
    }
}

impl BfxClientError {
    pub(crate) fn from_auth_error(code: u32, desc: String) -> Self {
        match code {
            10020 => Self::RequestParametersError { code, desc },
            10100 => Self::InvalidCredentials { code, desc },
            _ => Self::UnexpectedResponse {
                event: Some("error".to_string()),
                code: Some(code),
                desc,
            },
        }
    }
}

impl ErrorSeverity for BfxClientError {
    fn severity(&self) -> Level {
        match self {
            Self::Reqwest(_) => Level::WARN,
            Self::Deserialization(_) => Level::ERROR,
            Self::ConversionError(e) => e.severity(),
            Self::RequestParametersError { .. } => Level::ERROR,
            Self::InvalidCredentials { .. } => Level::ERROR,
            Self::UnexpectedResponse { .. } => Level::ERROR,
            Self::UnexpectedHttpStatus { .. } => Level::ERROR,
            Self::InvalidNotificationStatus { .. } => Level::ERROR,
        }
    }
}
