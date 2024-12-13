use thiserror::Error;

use crate::primitives::WithdrawalId;

#[derive(Error, Debug)]
pub enum WithdrawalError {
    #[error("WithdrawalError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WithdrawalError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("WithdrawalError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("WithdrawalError - AlreadyConfirmed: {0}")]
    AlreadyConfirmed(WithdrawalId),
    #[error("WithdrawalError - AlreadyCancelled: {0}")]
    AlreadyCancelled(WithdrawalId),
    #[error("WithdrawalError - NotApproved: {0}")]
    NotApproved(WithdrawalId),
}

es_entity::from_es_entity_error!(WithdrawalError);
