use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreCreditError {
    #[error("CoreCreditError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreCreditError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
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
    #[error("CoreCreditError - InterestAccrualError: {0}")]
    InterestAccrualError(#[from] super::interest_accrual::error::InterestAccrualError),
    #[error("CoreCreditError - PriceError: {0}")]
    PriceError(#[from] core_price::error::PriceError),
    #[error("CoreCreditError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("CoreCreditError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CoreCreditError - CreditRecipientMismatchForCreditFacility")]
    CreditRecipientMismatchForCreditFacility,
    #[error("CreditFacilityError - SubjectIsNotCreditRecipientId")]
    SubjectIsNotCreditRecipient,
}

es_entity::from_es_entity_error!(CoreCreditError);
