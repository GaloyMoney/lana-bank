use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("ApplicationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ApplicationError - MigrateError: {0}")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("ApplicationError - JobError: {0}")]
    JobError(#[from] crate::job::error::JobError),
    #[error("ApplicationError - CustomerError: {0}")]
    CustomerError(#[from] crate::customer::error::CustomerError),
    #[error("ApplicationError - CustomerSyncError: {0}")]
    CustomerSyncError(#[from] customer_sync::error::CustomerSyncError),
    #[error("ApplicationError - DepositSyncError: {0}")]
    DepositSyncError(#[from] deposit_sync::error::DepositSyncError),
    #[error("ApplicationError - NotificationError: {0}")]
    NotificationError(#[from] notification::error::NotificationError),
    #[error("ApplicationError - CreditFacilityError: {0}")]
    CreditFacilityError(#[from] crate::credit::error::CoreCreditError),
    #[error("ApplicationError - CreditLedgerError: {0}")]
    CreditLedgerError(#[from] core_credit::CreditLedgerError),
    #[error("ApplicationError - CollateralLedgerError: {0}")]
    CollateralLedgerError(#[from] core_credit::CollateralLedgerError),
    #[error("ApplicationError - CollectionLedgerError: {0}")]
    CollectionLedgerError(#[from] core_credit::CollectionLedgerError),
    #[error("ApplicationError - TrialBalanceError: {0}")]
    TrialBalanceError(#[from] crate::trial_balance::error::TrialBalanceError),
    #[error("ApplicationError - ProfitAndLossStatementError: {0}")]
    ProfitAndLossStatementError(#[from] crate::profit_and_loss::error::ProfitAndLossStatementError),
    #[error("ApplicationError - BalanceSheetError: {0}")]
    BalanceSheetError(#[from] crate::balance_sheet::error::BalanceSheetError),
    #[error("ApplicationError - CoreAccessError: {0}")]
    CoreAccessError(#[from] crate::access::error::CoreAccessError),
    #[error("ApplicationError - UserOnboardingError: {0}")]
    UserOnboardingError(#[from] user_onboarding::error::UserOnboardingError),
    #[error("ApplicationError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("ApplicationError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::DomainConfigError),
    #[error("ApplicationError - AuditError: {0}")]
    AuditError(#[from] crate::audit::error::AuditError),
    #[error("ApplicationError - PriceError: {0}")]
    PriceError(#[from] crate::price::error::PriceError),
    #[error("ApplicationError - TimeEventsError: {0}")]
    TimeEventsError(#[from] crate::time_events::error::TimeEventsError),
    #[error("ApplicationError - AccountingInitError: {0}")]
    AccountingInitError(#[from] crate::accounting_init::error::AccountingInitError),
    #[error("ApplicationError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("ApplicationError - DashboardError: {0}")]
    DashboardError(#[from] dashboard::error::DashboardError),
    #[error("ApplicationError - CalaInit: {0}")]
    CalaError(#[from] cala_ledger::error::LedgerError),
    #[error("ApplicationError - ChartOfAccountsError: {0}")]
    ChartOfAccountsError(#[from] core_accounting::chart_of_accounts::error::ChartOfAccountsError),
    #[error("ApplicationError - DepositError: {0}")]
    DepositError(#[from] crate::deposit::error::CoreDepositError),
    #[error("ApplicationError - StorageError: {0}")]
    StorageError(#[from] crate::storage::error::StorageError),
    #[error("ApplicationError - KycError: {0}")]
    KycError(#[from] crate::kyc::error::KycError),
    #[error("ApplicationError - CustodyError: {0}")]
    CustodyError(#[from] crate::custody::error::CoreCustodyError),
    #[error("ApplicationError - ContractCreationError: {0}")]
    ContractCreationError(#[from] crate::contract_creation::ContractCreationError),
    #[error("ApplicationError - ReportError: {0}")]
    ReportError(#[from] crate::report::error::ReportError),
    #[error("ApplicationError - TracingError: {0}")]
    TracingError(#[from] tracing_utils::TracingError),
    #[error("ApplicationError - CanNotCreateProposalForClosedOrFrozenAccount")]
    CanNotCreateProposalForClosedOrFrozenAccount,
    #[error("ApplicationError - ClosedOrFrozenAccount")]
    ClosedOrFrozenAccount,
}

impl ErrorSeverity for ApplicationError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::MigrateError(_) => Level::ERROR,
            Self::JobError(_) => Level::ERROR,
            Self::CustomerError(e) => e.severity(),
            Self::CustomerSyncError(e) => e.severity(),
            Self::DepositSyncError(e) => e.severity(),
            Self::NotificationError(e) => e.severity(),
            Self::CreditFacilityError(e) => e.severity(),
            Self::CreditLedgerError(_) => Level::ERROR,
            Self::CollateralLedgerError(_) => Level::ERROR,
            Self::CollectionLedgerError(_) => Level::ERROR,
            Self::TrialBalanceError(e) => e.severity(),
            Self::ProfitAndLossStatementError(e) => e.severity(),
            Self::BalanceSheetError(e) => e.severity(),
            Self::CoreAccessError(e) => e.severity(),
            Self::UserOnboardingError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::DomainConfigError(e) => e.severity(),
            Self::AuditError(e) => e.severity(),
            Self::PriceError(e) => e.severity(),
            Self::AccountingInitError(e) => e.severity(),
            Self::GovernanceError(e) => e.severity(),
            Self::DashboardError(e) => e.severity(),
            Self::CalaError(_) => Level::ERROR,
            Self::ChartOfAccountsError(e) => e.severity(),
            Self::DepositError(e) => e.severity(),
            Self::StorageError(e) => e.severity(),
            Self::KycError(e) => e.severity(),
            Self::CustodyError(e) => e.severity(),
            Self::ContractCreationError(e) => e.severity(),
            Self::ReportError(e) => e.severity(),
            Self::TracingError(e) => e.severity(),
            Self::CanNotCreateProposalForClosedOrFrozenAccount => Level::WARN,
            Self::ClosedOrFrozenAccount => Level::WARN,
            Self::TimeEventsError(e) => e.severity(),
        }
    }
}
