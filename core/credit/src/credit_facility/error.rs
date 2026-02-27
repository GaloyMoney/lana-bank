use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use money::{Satoshis, UsdCents};

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
    #[error("CreditFacilityError - InterestAccrualCycleError: {0}")]
    InterestAccrualCycleError(
        #[from] crate::credit_facility::interest_accrual_cycle::error::InterestAccrualCycleError,
    ),
    #[error("CreditFacilityError - DisbursalError: {0}")]
    DisbursalError(#[from] crate::disbursal::error::DisbursalError),
    #[error("CreditFacilityError - ApprovalInProgress")]
    ApprovalInProgress,
    #[error("CreditFacilityError - Denied")]
    Denied,
    #[error("CreditFacilityError - DisbursalPastMaturityDate")]
    DisbursalPastMaturityDate,
    #[error("CreditFacilityError - OnlyOneDisbursalAllowed")]
    OnlyOneDisbursalAllowed,
    #[error("CreditFacilityError - NotActivatedYet")]
    NotActivatedYet,
    #[error("CreditFacilityError - PaymentBeforeFacilityActivation")]
    PaymentBeforeFacilityActivation,
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
    #[error("CreditFacilityError - InterestAccrualCycleWithInvalidFutureStartDate")]
    InterestAccrualCycleWithInvalidFutureStartDate,
    #[error("CreditFacilityError - InProgressInterestAccrualCycleNotCompletedYet")]
    InProgressInterestAccrualCycleNotCompletedYet,
    #[error(
        "CreditFacilityError - DisbursalAmountTooLarge: amount '{0}' is larger than facility balance '{1}'"
    )]
    DisbursalAmountTooLarge(UsdCents, UsdCents),
    #[error(
        "CreditFacilityError - NoSuchLiquidationInitiated: liquidation {0} attempted to complete but has not been initiated"
    )]
    NoSuchLiquidationInitiated(core_credit_collateral::LiquidationId),
    #[error("CreditFacilityError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CreditFacilityError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CreditFacilityError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::CreditLedgerError),
    #[error("CreditFacilityError - PriceError: {0}")]
    PriceError(#[from] core_price::error::PriceError),
    #[error("CreditFacilityError - ObligationError: {0}")]
    ObligationError(#[from] core_credit_collection::ObligationError),
    #[error("CreditFacilityError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("CreditFacilityError - PublicIdError: {0}")]
    PublicIdError(#[from] public_id::PublicIdError),
    #[error("CreditFacilityError - PaymentAllocationError: {0}")]
    PaymentAllocationError(#[from] core_credit_collection::PaymentAllocationError),
    #[error("CreditFacilityError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CreditFacilityError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("CreditFacilityError - CreditFacilityProposalError: {0}")]
    CreditFacilityProposalError(
        #[from] crate::pending_credit_facility::error::PendingCreditFacilityError,
    ),
    #[error("CreditFacilityError - CollateralError: {0}")]
    CollateralError(#[from] core_credit_collateral::error::CollateralError),
}

impl ErrorSeverity for CreditFacilityError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntityError(e) => e.severity(),
            Self::CursorDestructureError(_) => Level::ERROR,
            Self::ConversionError(e) => e.severity(),
            Self::InterestAccrualCycleError(e) => e.severity(),
            Self::DisbursalError(e) => e.severity(),
            Self::ApprovalInProgress => Level::WARN,
            Self::Denied => Level::WARN,
            Self::DisbursalPastMaturityDate => Level::WARN,
            Self::OnlyOneDisbursalAllowed => Level::WARN,
            Self::NotActivatedYet => Level::WARN,
            Self::PaymentBeforeFacilityActivation => Level::WARN,
            Self::InterestAccrualNotCompletedYet => Level::WARN,
            Self::NoDisbursalInProgress => Level::WARN,
            Self::CollateralNotUpdated(_, _) => Level::ERROR,
            Self::NoCollateral => Level::WARN,
            Self::BelowMarginLimit => Level::WARN,
            Self::PaymentExceedsOutstandingCreditFacilityAmount(_, _) => Level::WARN,
            Self::FacilityLedgerBalanceMismatch => Level::ERROR,
            Self::OutstandingAmount => Level::WARN,
            Self::InterestAccrualCycleWithInvalidFutureStartDate => Level::ERROR,
            Self::InProgressInterestAccrualCycleNotCompletedYet => Level::WARN,
            Self::DisbursalAmountTooLarge(_, _) => Level::WARN,
            Self::NoSuchLiquidationInitiated(_) => Level::WARN,
            Self::AuthorizationError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::LedgerError(e) => e.severity(),
            Self::PriceError(e) => e.severity(),
            Self::ObligationError(e) => e.severity(),
            Self::GovernanceError(e) => e.severity(),
            Self::PublicIdError(e) => e.severity(),
            Self::PaymentAllocationError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::RegisterEventHandler(_) => Level::ERROR,
            Self::CreditFacilityProposalError(e) => e.severity(),
            Self::CollateralError(e) => e.severity(),
        }
    }
}

es_entity::from_es_entity_error!(CreditFacilityError);
