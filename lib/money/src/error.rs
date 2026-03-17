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

#[derive(Error, Debug)]
pub enum CurrencyBagError {
    #[error("CurrencyBagError - CurrencyNotSupported: {0}")]
    CurrencyNotSupported(&'static str),
    #[error("CurrencyBagError - InsufficientBalance: {0}")]
    InsufficientBalance(&'static str),
}

impl ErrorSeverity for CurrencyBagError {
    fn severity(&self) -> Level {
        match self {
            Self::CurrencyNotSupported(_) => Level::WARN,
            Self::InsufficientBalance(_) => Level::WARN,
        }
    }
}
