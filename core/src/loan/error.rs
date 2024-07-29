use thiserror::Error;

use crate::primitives::*;

#[derive(Error, Debug)]
pub enum LoanError {
    #[error("LoanError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LoanError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("LoanError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("LoanError - LoanTermsError: {0}")]
    LoanTermsError(#[from] super::terms::error::LoanTermsError),
    #[error("LoanError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error("LoanError - UserError: '{0}'")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("LoanError - UserNotFound: {0}")]
    CustomerNotFound(CustomerId),
    #[error("LoanError - UserNotAllowedToCreateLoan: {0}")]
    CustomerNotAllowedToCreateLoan(CustomerId),
    #[error("LoanError - InsufficientCollateral: {0} < {1}")]
    InsufficientCollateral(Satoshis, Satoshis),
    #[error("LoanError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("LoanError - AlreadyCompleted")]
    AlreadyCompleted,
    #[error("LoanError - AlreadyApproved")]
    AlreadyApproved,
    #[error("LoanError - PaymentExceedsOutstandingLoanAmount: {0} > {1}")]
    PaymentExceedsOutstandingLoanAmount(UsdCents, UsdCents),
    #[error("LoanError - TermsNotSet")]
    TermsNotSet,
    #[error("LoanError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
}
