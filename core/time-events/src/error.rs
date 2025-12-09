use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeEventsError {
    #[error("Invalid timezone")]
    InvalidTimezone { timezone: String },
    #[error("Invalid time format")]
    InvalidTimeFormat { time_format: String },
    #[error("Could not create datetime for closing time")]
    InvalidClosingDateTime { closing_time: String },
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}
