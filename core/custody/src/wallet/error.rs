use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use super::repo::{WalletCreateError, WalletFindError, WalletModifyError, WalletQueryError};

#[derive(thiserror::Error, Debug, strum::IntoStaticStr)]
pub enum WalletError {
    #[error("WalletError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WalletError - Create: {0}")]
    Create(#[from] WalletCreateError),
    #[error("WalletError - Modify: {0}")]
    Modify(#[from] WalletModifyError),
    #[error("WalletError - Find: {0}")]
    Find(#[from] WalletFindError),
    #[error("WalletError - Query: {0}")]
    Query(#[from] WalletQueryError),
}

impl ErrorSeverity for WalletError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
        }
    }
}
