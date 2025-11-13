use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreCreditError {
    #[error("CoreCreditError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreCreditError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreCreditError - CustomerError: {0}")]
    CustomerError(#[from] core_customer::error::CustomerError),
    #[error("CoreCreditError - DepositError: {0}")]
    DepositError(#[from] core_deposit::error::CoreDepositError),
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
    #[error("CoreCreditError - ObligationError: {0}")]
    ObligationError(#[from] super::obligation::error::ObligationError),
    #[error("CoreCreditError - InterestAccrualCycleError: {0}")]
    InterestAccrualCycleError(
        #[from] super::interest_accrual_cycle::error::InterestAccrualCycleError,
    ),
    #[error("CoreCreditError - PriceListenerError: {0}")]
    PriceListenerError(#[from] core_price_listener::error::PriceListenerError),
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
