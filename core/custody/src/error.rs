use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::custodian::error::{
    CustodianCreateError, CustodianFindError, CustodianModifyError, CustodianQueryError,
};
use crate::wallet::error::{
    WalletCreateError, WalletFindError, WalletModifyError, WalletQueryError,
};

#[derive(Error, Debug)]
pub enum CoreCustodyError {
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
    #[error("CoreCustodyError - JobError: {0}")]
    JobError(String),
}

impl From<CustodianCreateError> for CoreCustodyError {
    fn from(e: CustodianCreateError) -> Self {
        Self::Custodian(e.into())
    }
}

impl From<CustodianFindError> for CoreCustodyError {
    fn from(e: CustodianFindError) -> Self {
        Self::Custodian(e.into())
    }
}

impl From<CustodianModifyError> for CoreCustodyError {
    fn from(e: CustodianModifyError) -> Self {
        Self::Custodian(e.into())
    }
}

impl From<CustodianQueryError> for CoreCustodyError {
    fn from(e: CustodianQueryError) -> Self {
        Self::Custodian(e.into())
    }
}

impl From<WalletCreateError> for CoreCustodyError {
    fn from(e: WalletCreateError) -> Self {
        Self::Wallet(e.into())
    }
}

impl From<WalletFindError> for CoreCustodyError {
    fn from(e: WalletFindError) -> Self {
        Self::Wallet(e.into())
    }
}

impl From<WalletModifyError> for CoreCustodyError {
    fn from(e: WalletModifyError) -> Self {
        Self::Wallet(e.into())
    }
}

impl From<WalletQueryError> for CoreCustodyError {
    fn from(e: WalletQueryError) -> Self {
        Self::Wallet(e.into())
    }
}

impl ErrorSeverity for CoreCustodyError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::Sqlx(_) => Level::ERROR,
            Self::Custodian(e) => e.severity(),
            Self::CustodianClient(e) => e.severity(),
            Self::Wallet(e) => e.severity(),
            Self::InboxError(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
        }
    }
}
