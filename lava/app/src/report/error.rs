use thiserror::Error;

use crate::primitives::ReportId;

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("ReportError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ReportError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("ReportError - DataformCompilation: {0}")]
    DataformCompilation(String),
    #[error("ReportError - DataformInvocation: {0}")]
    DataformInvocation(String),
    #[error("ReportError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("ReportError - GCPAuth: {0}")]
    GCPAuth(#[from] gcp_auth::Error),
    #[error("ReportError - BigQuery: {0}")]
    BigQuery(#[from] gcp_bigquery_client::error::BQError),
    #[error("ReportError - Base64Decode: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("ReportError - FromUtf8Error : {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("ReportError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("ReportError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("ReportError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("ReportError - CouldNotFindById: {0}")]
    CouldNotFindById(ReportId),
    #[error("ReportError - StorageError: {0}")]
    StorageError(#[from] crate::storage::StorageError),
}
