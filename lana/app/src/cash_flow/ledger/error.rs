use thiserror::Error;

#[derive(Error, Debug)]
pub enum CashFlowStatementLedgerError {
    #[error("CashFlowStatementLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CashFlowStatementLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("CashFlowStatementLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("CashFlowStatementLedgerError - CalaBalance: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("CashFlowStatementError - ConversionError: {0}")]
    Statement(#[from] crate::statement::error::StatementError),
    #[error("CashFlowStatementLedgerError - NonAccountSetMemberTypeFound")]
    NonAccountSetMemberTypeFound,
    #[error("CashFlowStatementLedgerError - NotFound: {0}")]
    NotFound(String),
}

impl CashFlowStatementLedgerError {
    pub fn account_set_exists(&self) -> bool {
        matches!(
            self,
            Self::CalaAccountSet(
                cala_ledger::account_set::error::AccountSetError::ExternalIdAlreadyExists,
            )
        )
    }
}
