use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum DepositAccountError {
    #[error("DepositAccountError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositAccountError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DepositAccountError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DepositAccountError - CannotFreezeInactiveAccount: {0}")]
    CannotFreezeInactiveAccount(crate::DepositAccountId),
    #[error("DepositAccountError - CannotUpdateClosedAccount: {0}")]
    CannotUpdateClosedAccount(crate::DepositAccountId),
    #[error("DepositAccountError - CannotUpdateFrozenAccount")]
    CannotUpdateFrozenAccount(crate::DepositAccountId),
    #[error("DepositAccountError - BalanceIsNotZero")]
    BalanceIsNotZero,
}

es_entity::from_es_entity_error!(DepositAccountError);

impl ErrorSeverity for DepositAccountError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::CannotFreezeInactiveAccount(_) => Level::WARN,
            Self::CannotUpdateClosedAccount(_) => Level::WARN,
            Self::CannotUpdateFrozenAccount(_) => Level::WARN,
            Self::BalanceIsNotZero => Level::WARN,
        }
    }
}
