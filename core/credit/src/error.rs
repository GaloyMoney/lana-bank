use thiserror::Error;

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
    #[error("CoreCreditError - EsEntityError: {0}")]
    EsEntityError(es_entity::EsEntityError),
    #[error("CoreCreditError - PaymentError: {0}")]
    CoreCreditError(#[from] super::credit_facility::error::CreditFacilityError),
    #[error("CoreCreditError - PaymentError: {0}")]
    PaymentError(#[from] super::payment::error::PaymentError),
    #[error("CoreCreditError - DisbursalError: {0}")]
    DisbursalError(#[from] super::disbursal::error::DisbursalError),
    #[error("CoreCreditError - InterestAccrualCycleError: {0}")]
    InterestAccrualCycleError(
        #[from] super::interest_accrual_cycle::error::InterestAccrualCycleError,
    ),
    #[error("CoreCreditError - PriceError: {0}")]
    PriceError(#[from] core_price::error::PriceError),
    #[error("CoreCreditError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("CoreCreditError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError),
    #[error("CoreCreditError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CoreCreditError - CustomerMismatchForCreditFacility")]
    CustomerMismatchForCreditFacility,
    #[error("CoreCreditError - SubjectIsNotCustomer")]
    SubjectIsNotCustomer,
    #[error("CoreCreditError - CustomerIsNotActive")]
    CustomerNotActive,
    #[error("CoreCreditError - CustomerNotFound")]
    CustomerNotFound,
    #[error("CoreCreditError ChartIdMismatch")]
    ChartIdMismatch,
    #[error("CoreCreditError - CreditConfigAlreadyExists")]
    CreditConfigAlreadyExists,
}

es_entity::from_es_entity_error!(CoreCreditError);
