use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreCustodyError {
    #[error("CoreCustodyError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),

    #[error("CoreCustodyError - CustodianConfigError: {0}")]
    CustodianConfig(#[from] crate::custodian_config::error::CustodianConfigError),
}
