use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use super::repo::{
    ChartCreateError, ChartFindError, ChartModifyError, ChartNodeCreateError, ChartNodeFindError,
    ChartNodeModifyError, ChartNodeQueryError, ChartQueryError,
};

#[derive(Error, Debug)]
pub enum ChartOfAccountsError {
    #[error("ChartOfAccountsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ChartOfAccountsError - Create: {0}")]
    Create(#[from] ChartCreateError),
    #[error("ChartOfAccountsError - Modify: {0}")]
    Modify(#[from] ChartModifyError),
    #[error("ChartOfAccountsError - Find: {0}")]
    Find(#[from] ChartFindError),
    #[error("ChartOfAccountsError - Query: {0}")]
    Query(#[from] ChartQueryError),
    #[error("ChartOfAccountsError - ChartNodeCreate: {0}")]
    ChartNodeCreate(#[from] ChartNodeCreateError),
    #[error("ChartOfAccountsError - ChartNodeModify: {0}")]
    ChartNodeModify(#[from] ChartNodeModifyError),
    #[error("ChartOfAccountsError - ChartNodeFind: {0}")]
    ChartNodeFind(#[from] ChartNodeFindError),
    #[error("ChartOfAccountsError - ChartNodeQuery: {0}")]
    ChartNodeQuery(#[from] ChartNodeQueryError),
    #[error("ChartOfAccountsError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("ChartOfAccountsError - CodeNotFoundInChart: {0}")]
    CodeNotFoundInChart(crate::primitives::AccountCode),
    #[error("ChartOfAccountsError - CsvParseError: {0}")]
    CsvParse(#[from] super::CsvParseError),
    #[error("ChartOfAccountsError - CalaLedgerError: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ChartOfAccountsError - CalaAccountError: {0}")]
    CalaAccount(#[from] cala_ledger::account::error::AccountError),
    #[error("ChartOfAccountsError - CalaAccountSetError: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("ChartOfAccountsError - ChartLedgerError: {0}")]
    ChartLedgerError(#[from] super::ledger::error::ChartLedgerError),
    #[error("ChartOfAccountsError - AccountCodeError: {0}")]
    AccountCode(#[from] crate::primitives::AccountCodeError),
    #[error("ChartOfAccountsError - NonLeafAccount: {0}")]
    NonLeafAccount(String),
    #[error("ChartOfAccountsError - ParentAccountNotFound: {0}")]
    ParentAccountNotFound(String),
    #[error("ChartOfAccountsError - ChartOfAccountsNotFoundByReference: {0}")]
    ChartOfAccountsNotFoundByReference(String),
    #[error("ChartOfAccountsError - AccountCodeHasInvalidParent: {0}")]
    AccountCodeHasInvalidParent(String),
    #[error("ChartOfAccountsError - AccountingBaseConfigError: {0}")]
    AccountingBaseConfigError(#[from] crate::primitives::AccountingBaseConfigError),
    #[error("ChartOfAccountsError - BaseConfigAlreadyInitializedWithDifferentValues")]
    BaseConfigAlreadyInitializedWithDifferentConfig,
    #[error("ChartOfAccountsError - BaseConfigNotInitialized")]
    BaseConfigNotInitialized,
    #[error("ChartOfAccountsError - AccountCategoryNotSupported: {0:?}")]
    AccountCategoryNotSupported(crate::primitives::AccountCategory),
}

impl ErrorSeverity for ChartOfAccountsError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::Create(_) => Level::ERROR,
            Self::Modify(_) => Level::ERROR,
            Self::Find(_) => Level::ERROR,
            Self::Query(_) => Level::ERROR,
            Self::ChartNodeCreate(_) => Level::ERROR,
            Self::ChartNodeModify(_) => Level::ERROR,
            Self::ChartNodeFind(_) => Level::ERROR,
            Self::ChartNodeQuery(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::CodeNotFoundInChart(_) => Level::WARN,
            Self::CsvParse(e) => e.severity(),
            Self::CalaLedger(_) => Level::ERROR,
            Self::CalaAccount(_) => Level::ERROR,
            Self::CalaAccountSet(_) => Level::ERROR,
            Self::ChartLedgerError(e) => e.severity(),
            Self::AccountCode(e) => e.severity(),
            Self::NonLeafAccount(_) => Level::WARN,
            Self::ParentAccountNotFound(_) => Level::ERROR,
            Self::ChartOfAccountsNotFoundByReference(_) => Level::ERROR,
            Self::AccountCodeHasInvalidParent(_) => Level::ERROR,
            Self::AccountingBaseConfigError(e) => e.severity(),
            Self::BaseConfigAlreadyInitializedWithDifferentConfig => Level::WARN,
            Self::BaseConfigNotInitialized => Level::ERROR,
            Self::AccountCategoryNotSupported(_) => Level::WARN,
        }
    }
}
