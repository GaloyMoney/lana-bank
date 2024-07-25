use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("DomainError - DecimalError: {0}")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("DomainError - UnexpectedNegativeNumber: {0}")]
    UnexpectedNegativeNumber(rust_decimal::Decimal),
}
