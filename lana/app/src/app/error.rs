use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("ApplicationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ApplicationError - MigrateError: {0}")]
    MigateError(#[from] sqlx::migrate::MigrateError),
    #[error("ApplicationError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("ApplicationError - CustomerError: {0}")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("ApplicationError - CustomerOnboardingError: {0}")]
    CustomerOnboardingError(#[from] customer_onboarding::error::CustomerOnboardingError),
    #[error("ApplicationError - CreditFacilityError: {0}")]
    CreditFacilityError(#[from] crate::credit_facility::error::CreditFacilityError),
    #[error("ApplicationError - TrialBalanceError: {0}")]
    TrialBalanceError(#[from] crate::trial_balance::error::TrialBalanceError),
    #[error("ApplicationError - ProfitAndLossStatementError: {0}")]
    ProfitAndLossStatementError(#[from] crate::profit_and_loss::error::ProfitAndLossStatementError),
    #[error("ApplicationError - BalanceSheetError: {0}")]
    BalanceSheetError(#[from] crate::balance_sheet::error::BalanceSheetError),
    #[error("ApplicationError - UserError: {0}")]
    UserError(#[from] crate::user::error::UserError),
    #[error("ApplicationError - UserOnboardingError: {0}")]
    UserOnboardingError(#[from] user_onboarding::error::UserOnboardingError),
    #[error("ApplicationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("ApplicationError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("ApplicationError - ReportError: {0}")]
    ReportError(#[from] crate::report::error::ReportError),
    #[error("ApplicationError - PriceError: {0}")]
    PriceError(#[from] crate::price::error::PriceError),
    #[error("ApplicationError - AccountingInitError: {0}")]
    AccountingInitError(#[from] crate::accounting_init::error::AccountingInitError),
    #[error("ApplicationError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("ApplicationError - DashboardError: {0}")]
    DashboardError(#[from] dashboard::error::DashboardError),
    #[error("ApplicationError - CalaInit: {0}")]
    CalaError(#[from] cala_ledger::error::LedgerError),
    #[error("ApplicationError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountsError),
    #[error("ApplicationError - DepositError: {0}")]
    DepositError(#[from] crate::deposit::error::CoreDepositError),
}
