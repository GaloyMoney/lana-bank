use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustodianConfigError {
    #[error("CustodianConfigError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CustodianConfigError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("RoleError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
}

es_entity::from_es_entity_error!(CustodianConfigError);
