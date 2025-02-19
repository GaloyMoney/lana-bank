use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("ReportError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ReportError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("ReportError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("ReportError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
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
    #[error("ReportError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("ReportError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("ReportError - StorageError: {0}")]
    StorageError(#[from] crate::storage::StorageError),
    #[error("ReportError - ServiceAccountError: {0}")]
    ServiceAccountError(#[from] crate::service_account::error::ServiceAccountError),
}

es_entity::from_es_entity_error!(ReportError);
