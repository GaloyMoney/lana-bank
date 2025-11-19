use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositAccountError {
    #[error("DepositAccountError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositAccountError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DepositAccountError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DepositAccountError - CannotFreezeInActiveAccount: {0}")]
    CannotFreezeInActiveAccount(crate::DepositAccountId),
    #[error("DepositAccountError - CannotUnfreezeNonFrozenAccount: {0}")]
    CannotUnfreezeNonFrozenAccount(crate::DepositAccountId),
    #[error("DepositAccountError - CannotUpdateClosedAccount: {0}")]
    CannotUpdateClosedAccount(crate::DepositAccountId),
    #[error("DepositAccountError - CannotUpdateFrozenAccount")]
    CannotUpdateFrozenAccount(crate::DepositAccountId),
    #[error("DepositAccountError - BalanceIsNotZero")]
    BalanceIsNotZero,
    #[error("DepositAccountError - CannotCloseAccount")]
    CannotCloseAccount,
    #[error("DepositAccountError - CannotCloseOrFreezeAccount")]
    CannotCloseOrFreezeAccount,
}

es_entity::from_es_entity_error!(DepositAccountError);
