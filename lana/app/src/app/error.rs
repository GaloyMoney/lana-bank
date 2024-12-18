use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("ApplicationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ApplicationError - MigrateError: {0}")]
    MigateError(#[from] sqlx::migrate::MigrateError),
    #[error("ApplicationError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("ApplicationError - LedgerError: {0}")]
    LedgerError(#[from] crate::ledger::error::LedgerError),
    #[error("ApplicationError - CustomerError: {0}")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("ApplicationError - CreditFacilityError: {0}")]
    CreditFacilityError(#[from] crate::credit_facility::error::CreditFacilityError),
    #[error("ApplicationError - UserError: {0}")]
    UserError(#[from] crate::user::error::UserError),
    #[error("ApplicationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("ApplicationError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("ApplicationError - ReportError: {0}")]
    ReportError(#[from] crate::report::error::ReportError),
    #[error("ApplicationError - PriceError: {0}")]
    PriceError(#[from] crate::price::error::PriceError),
    #[error("ApplicationError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("ApplicationError - DashboardError: {0}")]
    DashboardError(#[from] dashboard::error::DashboardError),
    #[error("ApplicationError - CalaInit: {0}")]
    CalaError(#[from] cala_ledger::error::LedgerError),
    #[error("ApplicationError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountError),
    #[error("ApplicationError - DepositError: {0}")]
    DepositError(#[from] crate::deposit::error::CoreDepositError),
    #[error("ApplicationError - JournalError: {0}")]
    JournalError(#[from] cala_ledger::journal::error::JournalError),
}
