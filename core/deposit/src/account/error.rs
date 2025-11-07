use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositAccountError {
    #[error("CommitteeError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CommitteeError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CommitteeError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DepositAccountError - CannotFreezeNonActiveAccount: {0}")]
    CannotFreezeNonActiveAccount(crate::DepositAccountId),
    #[error("DepositAccountError - CannotUnfreezeNonFrozenAccount: {0}")]
    CannotUnfreezeNonFrozenAccount(crate::DepositAccountId),
}

es_entity::from_es_entity_error!(DepositAccountError);
