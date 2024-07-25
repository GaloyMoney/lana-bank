use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoanTermsError {
    #[error("LoanTermsError - DomainError: {0}")]
    DomainError(#[from] crate::error::DomainError),
}
