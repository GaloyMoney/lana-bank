use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceAccountError {
    #[error("ServiceAccountError - CLoudStorage: {0}")]
    CloudStorage(#[from] cloud_storage::Error),
    #[error("ServiceAccountError - Utf8Error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("ServiceAccountError - ProjectIdMissing")]
    GCPProjectIdMissing,
    #[error("ServiceAccountError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("ServiceAccountError - Base64Decode: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("ServiceAccountError - CredentialsNotProvided")]
    CredentialsNotProvided,
}
