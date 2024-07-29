use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoanTermsError {
    #[error("LoanTermsError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
}
