use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    DepositAccountCreateError, DepositAccountFindError, DepositAccountModifyError,
    DepositAccountQueryError,
};

#[derive(Error, Debug)]
pub enum DepositAccountError {
    #[error("DepositAccountError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositAccountError - Create: {0}")]
    Create(#[from] DepositAccountCreateError),
    #[error("DepositAccountError - Modify: {0}")]
    Modify(#[from] DepositAccountModifyError),
    #[error("DepositAccountError - Find: {0}")]
    Find(#[from] DepositAccountFindError),
    #[error("DepositAccountError - Query: {0}")]
    Query(#[from] DepositAccountQueryError),
    #[error("DepositAccountError - CannotUpdateClosedAccount: {0}")]
    CannotUpdateClosedAccount(crate::DepositAccountId),
    #[error("DepositAccountError - CannotUpdateFrozenAccount")]
    CannotUpdateFrozenAccount(crate::DepositAccountId),
    #[error("DepositAccountError - BalanceIsNotZero")]
    BalanceIsNotZero,
}

impl ErrorSeverity for DepositAccountError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::CannotUpdateClosedAccount(_) => Level::WARN,
            Self::CannotUpdateFrozenAccount(_) => Level::WARN,
            Self::BalanceIsNotZero => Level::WARN,
        }
    }
}
