use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeEventsError {
    #[error("Invalid timezone: {0}")]
    InvalidTimezone(String),

    #[error("Invalid time format: {0}")]
    InvalidTimeFormat(String),

    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}
