use thiserror::Error;

use crate::primitives::*;

#[derive(Error, Debug)]
pub enum LoanError {
    #[error("LoanError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("LoanError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("FixedTermLoanError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("LoanError - UserError: '{0}'")]
    UserError(#[from] crate::user::error::UserError),
    #[error("LoanError - UserNotFound: {0}")]
    UserNotFound(UserId),
    #[error("LoanError - UserNotAllowedToCreateLoan: {0}")]
    UserNotAllowedToCreateLoan(UserId),
    #[error("LoanError - InsufficientCollateral: {0} < {1}")]
    InsufficientCollateral(Satoshis, Satoshis),
    #[error("LoanError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("LoanError - AlreadyCompleted")]
    AlreadyCompleted,
    #[error("FixedTermLoanError - PaymentExceedsOutstandingLoanAmount: {0} > {1}")]
    PaymentExceedsOutstandingLoanAmount(UsdCents, UsdCents),
}
