use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("ConversionError - DecimalError: {0}")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("ConversionError - UnexpectedNegativeNumber: {0}")]
    UnexpectedNegativeNumber(rust_decimal::Decimal),
    #[error("ConversionError - Overflow")]
    Overflow,
    #[error("ConversionError - PrecisionLoss: {0} has fractional minor units")]
    PrecisionLoss(rust_decimal::Decimal),
}

impl ErrorSeverity for ConversionError {
    fn severity(&self) -> Level {
        match self {
            Self::DecimalError(_) => Level::ERROR,
            Self::UnexpectedNegativeNumber(_) => Level::WARN,
            Self::Overflow => Level::ERROR,
            Self::PrecisionLoss(_) => Level::WARN,
        }
    }
}
