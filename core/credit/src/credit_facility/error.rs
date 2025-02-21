use thiserror::Error;

use core_money::{Satoshis, UsdCents};

#[derive(Error, Debug)]
pub enum CreditFacilityError {
    #[error("CreditFacilityError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("FacilityError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CreditFacilityError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error("CreditFacilityError - InterestAccrualError: {0}")]
    InterestAccrualError(#[from] crate::interest_accrual::error::InterestAccrualError),
    #[error("CreditFacilityError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("CreditFacilityError - Denied")]
    Denied,
    #[error("CreditFacilityError - DisbursalExpiryDate")]
    DisbursalPastExpiryDate,
    #[error("CreditFacilityError - NotActivatedYet")]
    NotActivatedYet,
    #[error("CreditFacilityError - InterestAccrualNotCompletedYet")]
    InterestAccrualNotCompletedYet,
    #[error("CreditFacilityError - NoDisbursalInProgress")]
    NoDisbursalInProgress,
    #[error("CreditFacilityError - CollateralNotUpdated: before({0}), after({1})")]
    CollateralNotUpdated(Satoshis, Satoshis),
    #[error("CreditFacilityError - NoCollateral")]
    NoCollateral,
    #[error("CreditFacilityError - BelowMarginLimit")]
    BelowMarginLimit,
    #[error("CreditFacilityError - PaymentExceedsOutstandingCreditFacilityAmount: {0} > {1}")]
    PaymentExceedsOutstandingCreditFacilityAmount(UsdCents, UsdCents),
    #[error("CreditFacilityError - FacilityLedgerBalanceMismatch")]
    FacilityLedgerBalanceMismatch,
    #[error("CreditFacilityError - OutstandingAmount")]
    OutstandingAmount,
    #[error("CreditFacilityError - AlreadyCompleted")]
    AlreadyCompleted,
    #[error("CreditFacilityError - InterestAccrualInProgress")]
    InterestAccrualInProgress,
    #[error("CreditFacilityError - InterestAccrualWithInvalidFutureStartDate")]
    InterestAccrualWithInvalidFutureStartDate,
    #[error(
        "CreditFacilityError - DisbursalAmountTooLarge: amount '{0}' is larger than facility balance '{1}'"
    )]
    DisbursalAmountTooLarge(UsdCents, UsdCents),
}

es_entity::from_es_entity_error!(CreditFacilityError);
