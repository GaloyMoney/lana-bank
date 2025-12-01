use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::CVLPct;

#[derive(Error, Debug)]
pub enum TermsError {
    #[error("LoanTermsError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error(
        "LoanTermsError - InvalidFutureDateComparisonForAccrualDate: {1} is after accrual date {0}"
    )]
    InvalidFutureDateComparisonForAccrualDate(
        chrono::DateTime<chrono::Utc>,
        chrono::DateTime<chrono::Utc>,
    ),
    #[error("TermsError - MarginCallAboveInitialLimit: margin_call_cvl {0} >= initial_cvl {1}")]
    MarginCallAboveInitialLimit(CVLPct, CVLPct),
    #[error(
        "TermsError - MarginCallBelowLiquidationLimit: margin_call_cvl {0} <= liquidation_cvl {1}"
    )]
    MarginCallBelowLiquidationLimit(CVLPct, CVLPct),
    #[error("TermsError - UninitializedField: {0}")]
    UninitializedField(#[from] derive_builder::UninitializedFieldError),
}

impl ErrorSeverity for TermsError {
    fn severity(&self) -> Level {
        match self {
            Self::ConversionError(e) => e.severity(),
            Self::InvalidFutureDateComparisonForAccrualDate(_, _) => Level::WARN,
            Self::MarginCallAboveInitialLimit(_, _) => Level::WARN,
            Self::MarginCallBelowLiquidationLimit(_, _) => Level::WARN,
            Self::UninitializedField(_) => Level::ERROR,
        }
    }
}
