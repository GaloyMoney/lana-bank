use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreCustodyError {
    #[error("CoreCustodyError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreCustodyError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("CoreCustodyError - CustodianError: {0}")]
    Custodian(#[from] crate::custodian::error::CustodianError),
}
