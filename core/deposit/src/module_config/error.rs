use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositConfigError {
    #[error("DepositConfigError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("DepositConfigError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("DepositConfigError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("DepositConfigError - UninitializedField: {0}")]
    UninitializedField(#[from] derive_builder::UninitializedFieldError),
    #[error("DepositConfigError - ValuesNotConfigured")]
    ValuesNotConfigured,
}

es_entity::from_es_entity_error!(DepositConfigError);
