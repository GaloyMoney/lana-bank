use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreCreditError {
    #[error("CoreCreditError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreCreditError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreCreditError - CustomerError: {0}")]
    CustomerError(#[from] core_customer::error::CustomerError),
    #[error("CoreCreditError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreCreditError - CreditError: {0}")]
    CreditLedgerError(#[from] super::ledger::error::CreditLedgerError),
    #[error("CoreCreditError - ChartOfAccountsIntegrationError: {0}")]
    ChartOfAccountsIntegrationError(
        #[from] super::chart_of_accounts_integration::error::ChartOfAccountsIntegrationError,
    ),
    #[error("CoreCreditError - CreditFacilityProposalError: {0}")]
    CreditFacilityProposalError(
        #[from] super::credit_facility_proposal::error::CreditFacilityProposalError,
    ),
    #[error("CoreCreditError - PendingCreditFacilityError: {0}")]
    PendingCreditFacilityError(
        #[from] super::pending_credit_facility::error::PendingCreditFacilityError,
    ),
    #[error("CoreCreditError - CreditFacilityError: {0}")]
    CreditFacilityError(#[from] super::credit_facility::error::CreditFacilityError),
    #[error("CoreCreditError - HistoryError: {0}")]
    HistoryError(#[from] super::history::error::CreditFacilityHistoryError),
    #[error("CoreCreditError - RepaymentPlanError: {0}")]
    RepaymentPlanError(#[from] super::repayment_plan::error::CreditFacilityRepaymentPlanError),
    #[error("CoreCreditError - CollateralError: {0}")]
    CollateralError(#[from] super::collateral::error::CollateralError),
    #[error("CoreCreditError - CoreCustodyError: {0}")]
    CustodyError(#[from] core_custody::error::CoreCustodyError),
    #[error("CoreCreditError - PaymentError: {0}")]
    PaymentError(#[from] super::payment::error::PaymentError),
    #[error("CoreCreditError - PaymentAllocationError: {0}")]
    PaymentAllocationError(#[from] super::payment_allocation::error::PaymentAllocationError),
    #[error("CoreCreditError - DisbursalError: {0}")]
    DisbursalError(#[from] super::disbursal::error::DisbursalError),
    #[error("CoreCreditError - LiquidationError: {0}")]
    LiquidationError(#[from] super::liquidation::error::LiquidationError),
    #[error("CoreCreditError - ObligationError: {0}")]
    ObligationError(#[from] super::obligation::error::ObligationError),
    #[error("CoreCreditError - InterestAccrualCycleError: {0}")]
    InterestAccrualCycleError(
        #[from] super::interest_accrual_cycle::error::InterestAccrualCycleError,
    ),
    #[error("CoreCreditError - PriceError: {0}")]
    PriceError(#[from] core_price::error::PriceError),
    #[error("CoreCreditError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("CoreCreditError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CoreCreditError - CustomerMismatchForCreditFacility")]
    CustomerMismatchForCreditFacility,
    #[error("CoreCreditError - SubjectIsNotCustomer")]
    SubjectIsNotCustomer,
    #[error("CoreCreditError - CustomerIsNotVerified")]
    CustomerNotVerified,
    #[error("CoreCreditError - DisbursalBuilderError: {0}")]
    DisbursalBuilderError(#[from] super::NewDisbursalBuilderError),
    #[error("CoreCreditError - PublicIdError: {0}")]
    PublicIdError(#[from] public_id::PublicIdError),
}

impl ErrorSeverity for CoreCreditError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuditError(e) => e.severity(),
            Self::CustomerError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::CreditLedgerError(e) => e.severity(),
            Self::ChartOfAccountsIntegrationError(e) => e.severity(),
            Self::CreditFacilityProposalError(e) => e.severity(),
            Self::PendingCreditFacilityError(e) => e.severity(),
            Self::CreditFacilityError(e) => e.severity(),
            Self::HistoryError(e) => e.severity(),
            Self::RepaymentPlanError(e) => e.severity(),
            Self::CollateralError(e) => e.severity(),
            Self::CustodyError(e) => e.severity(),
            Self::PaymentError(e) => e.severity(),
            Self::PaymentAllocationError(e) => e.severity(),
            Self::DisbursalError(e) => e.severity(),
            Self::LiquidationError(e) => e.severity(),
            Self::ObligationError(e) => e.severity(),
            Self::InterestAccrualCycleError(e) => e.severity(),
            Self::PriceError(e) => e.severity(),
            Self::GovernanceError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::CustomerMismatchForCreditFacility => Level::ERROR,
            Self::SubjectIsNotCustomer => Level::WARN,
            Self::CustomerNotVerified => Level::WARN,
            Self::DisbursalBuilderError(_) => Level::ERROR,
            Self::PublicIdError(e) => e.severity(),
        }
    }
}
