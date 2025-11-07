use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositAccountError {
    #[error("DepositAccountError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositAccountError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DepositAccountError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DepositAccountError - CannotUpdateClosedDepositAccount")]
    CannotUpdateClosedDepositAccount,
    #[error("DepositAccountError - CannotCloseFrozenAccount")]
    CannotCloseFrozenAccount,
    #[error("DepositAccountError - BalanceIsNotZero")]
    BalanceIsNotZero,
}

es_entity::from_es_entity_error!(DepositAccountError);
