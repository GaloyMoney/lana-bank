use thiserror::Error;

use crate::primitives::{DepositAccountHolderId, Satoshis, UsdCents};

#[derive(Error, Debug)]
pub enum CreditFacilityError {
    #[error("CreditFacilityError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditFacilityError - CreditError: {0}")]
    CreditLedgerError(#[from] super::ledger::error::CreditLedgerError),
    #[error("CreditFacilityError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("FacilityError - CursorDestructureError: {0}")]
    CursorDestructureError(#[from] es_entity::CursorDestructureError),
    #[error("CreditFacilityError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("CreditFacilityError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("LoanError - PriceError: {0}")]
    PriceError(#[from] crate::price::error::PriceError),
    #[error("CreditFacilityError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("CreditFacilityError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("CreditFacilityError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error("CreditFacilityError - DisbursalError: {0}")]
    DisbursalError(#[from] super::disbursal::error::DisbursalError),
    #[error("CreditFacilityError - PaymentError: {0}")]
    PaymentError(#[from] super::payment::error::PaymentError),
    #[error("CreditFacilityError - InterestAccrualError: {0}")]
    InterestAccrualError(#[from] super::interest_accrual::error::InterestAccrualError),
    #[error("CreditFacilityError - DepositAccountForHolderNotFound: {0}")]
    DepositAccountForHolderNotFound(DepositAccountHolderId),
    #[error("CreditFacilityError - CoreDepositError: '{0}'")]
    CoreDepositError(#[from] crate::deposit::error::CoreDepositError),
    #[error("CreditFacilityError - UserError: '{0}'")]
    UserError(#[from] crate::user::error::UserError),
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
    #[error("CreditFacilityError - SubjectIsNotUser")]
    SubjectIsNotUser,
    #[error("CreditFacilityError - SubjectIsNotCustomer")]
    SubjectIsNotCustomer,
    #[error(
        "CreditFacilityError - DisbursalAmountTooLarge: amount '{0}' is larger than facility balance '{1}'"
    )]
    DisbursalAmountTooLarge(UsdCents, UsdCents),
    #[error("CreditFacilityError - CustomerMismatchForCreditFacility")]
    CustomerMismatchForCreditFacility,
}

es_entity::from_es_entity_error!(CreditFacilityError);
