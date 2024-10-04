use thiserror::Error;

use crate::primitives::{CustomerId, Satoshis, UsdCents};

#[derive(Error, Debug)]
pub enum CreditFacilityError {
    #[error("CreditFacilityError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("CreditFacilityError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("CreditFacilityError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("LoanError - PriceError: {0}")]
    PriceError(#[from] crate::price::error::PriceError),
    #[error("CreditFacilityError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("CreditFacilityError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error("CreditFacilityError - DisbursementError: {0}")]
    DisbursementError(#[from] super::disbursement::error::DisbursementError),
    #[error("CreditFacilityError - CustomerNotFound: {0}")]
    CustomerNotFound(CustomerId),
    #[error("CreditFacilityError - CustomerError: '{0}'")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("CreditFacilityError - UserError: '{0}'")]
    UserError(#[from] crate::user::error::UserError),
    #[error("CreditFacilityError - UserCannotApproveTwice")]
    UserCannotApproveTwice,
    #[error("CreditFacilityError - AlreadyApproved")]
    AlreadyApproved,
    #[error("CreditFacilityError - AlreadyExpired")]
    AlreadyExpired,
    #[error("CreditFacilityError - NoDisbursementInProgress")]
    NoDisbursementInProgress,
    #[error("CreditFacilityError - DisbursementInProgress")]
    DisbursementInProgress,
    #[error("CreditFacilityError - CollateralNotUpdated: before({0}), after({1})")]
    CollateralNotUpdated(Satoshis, Satoshis),
    #[error("CreditFacilityError - NoCollateral")]
    NoCollateral,
    #[error("CreditFacilityError - BelowMarginLimit")]
    BelowMarginLimit,
    #[error("CreditFacilityError - PaymentExceedsOutstandingCreditFacilityAmount: {0} > {1}")]
    PaymentExceedsOutstandingCreditFacilityAmount(UsdCents, UsdCents),
}
