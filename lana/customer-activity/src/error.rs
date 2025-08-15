use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerActivityError {
    #[error("CustomerActivityError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
