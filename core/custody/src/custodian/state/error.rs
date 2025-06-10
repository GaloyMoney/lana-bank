use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustodianStateError {
    #[error("CustodianStateError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
