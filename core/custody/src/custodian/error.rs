use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustodianError {
    #[error("CustodianError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustodianError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CustodianError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(CustodianError);
