use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreCustodyError {
    #[error("CustodianError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CoreCustodyError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreCustodyError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreCustodyError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreCustodyError - CustodianError: {0}")]
    Custodian(#[from] crate::custodian::error::CustodianError),
    #[error("CoreCustodyError - CustodianClientError: {0}")]
    CustodianClient(#[from] crate::custodian::client::error::CustodianClientError),
    #[error("CoreCustodyError - WalletError: {0}")]
    Wallet(#[from] crate::wallet::error::WalletError),
    #[error("CoreCustodyError - InboxError: {0}")]
    InboxError(#[from] obix::inbox::InboxError),
}

es_entity::from_es_entity_error!(CoreCustodyError);

impl ErrorSeverity for CoreCustodyError {
    fn severity(&self) -> Level {
        match self {
            Self::EsEntityError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
            Self::Custodian(e) => e.severity(),
            Self::CustodianClient(e) => e.severity(),
            Self::Wallet(e) => e.severity(),
            Self::InboxError(_) => Level::ERROR,
        }
    }
}
