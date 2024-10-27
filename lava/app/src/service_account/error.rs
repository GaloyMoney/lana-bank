use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceAccountError {
    #[error("Cloud Storage Error: {0}")]
    CloudStorage(#[from] cloud_storage::Error),
    #[error("ReportError - Utf8Error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("ReportError - ProjectIdMissing")]
    GCPProjectIdMissing,
    #[error("ReportError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("ReportError - Base64Decode: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}
