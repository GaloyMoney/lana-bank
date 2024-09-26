use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoanTermsError {
    #[error("LoanTermsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LoanError - TermsNotSet")]
    TermsNotSet,
    #[error("LoanTermsError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error(
        "LoanTermsError - InvalidFutureDateComparisonForAccrualDate: {1} is after accrual date {0}"
    )]
    InvalidFutureDateComparisonForAccrualDate(
        chrono::DateTime<chrono::Utc>,
        chrono::DateTime<chrono::Utc>,
    ),
}
