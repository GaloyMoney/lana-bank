use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use money::CurrencyCode;

use crate::ExchangeRateType;
use crate::provider::error::{
    PriceProviderCreateError, PriceProviderError, PriceProviderFindError, PriceProviderModifyError,
    PriceProviderQueryError,
};

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("PriceError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("PriceError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("PriceError - BfxClientError: {0}")]
    BfxClientError(#[from] bfx_client::BfxClientError),
    #[error("PriceError - ConversionError: {0}")]
    ConversionError(#[from] money::ConversionError),
    #[error("PriceError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("PriceError - Quote amount currency mismatch: expected {expected}, got {actual}")]
    QuoteAmountCurrencyMismatch {
        expected: CurrencyCode,
        actual: CurrencyCode,
    },
    #[error("PriceError - Price not yet available")]
    PriceUnavailable,
    #[error("PriceError - Unsupported exchange rate type: {rate_type}")]
    UnsupportedExchangeRateType { rate_type: ExchangeRateType },
    #[error(
        "PriceError - Unsupported exchange rate pair: base {base_currency}, quote {quote_currency}"
    )]
    UnsupportedExchangeRatePair {
        base_currency: CurrencyCode,
        quote_currency: CurrencyCode,
    },
    #[error("PriceError - PriceProviderError: {0}")]
    PriceProvider(#[from] PriceProviderError),
    #[error("PriceError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl From<PriceProviderCreateError> for PriceError {
    fn from(e: PriceProviderCreateError) -> Self {
        Self::PriceProvider(e.into())
    }
}

impl From<PriceProviderFindError> for PriceError {
    fn from(e: PriceProviderFindError) -> Self {
        Self::PriceProvider(e.into())
    }
}

impl From<PriceProviderModifyError> for PriceError {
    fn from(e: PriceProviderModifyError) -> Self {
        Self::PriceProvider(e.into())
    }
}

impl From<PriceProviderQueryError> for PriceError {
    fn from(e: PriceProviderQueryError) -> Self {
        Self::PriceProvider(e.into())
    }
}

impl ErrorSeverity for PriceError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::BfxClientError(e) => e.severity(),
            Self::ConversionError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::QuoteAmountCurrencyMismatch { .. } => Level::WARN,
            Self::PriceUnavailable => Level::WARN,
            Self::UnsupportedExchangeRateType { .. } => Level::WARN,
            Self::PriceProvider(e) => e.severity(),
            Self::UnsupportedExchangeRatePair { .. } => Level::WARN,
            Self::Sqlx(_) => Level::ERROR,
        }
    }
}
