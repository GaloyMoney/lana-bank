use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreStatementsError {
    #[error("CoreStatementsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
