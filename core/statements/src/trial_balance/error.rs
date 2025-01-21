use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrialBalanceStatementError {
    #[error("TrialBalanceStatementError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("TrialBalanceStatementError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("TrialBalanceStatementError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(TrialBalanceStatementError);
