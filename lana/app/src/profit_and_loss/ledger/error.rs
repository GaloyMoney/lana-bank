use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProfitAndLossStatementLedgerError {
    #[error("ProfitAndLossStatementLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ProfitAndLossStatementLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("ProfitAndLossStatementLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("ProfitAndLossStatementLedgerError - CalaBalance: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("ProfitAndLossStatementError - ConversionError: {0}")]
    Statement(#[from] crate::statement::error::StatementError),
    #[error("ProfitAndLossStatementLedgerError - NonAccountSetMemberTypeFound")]
    NonAccountSetMemberTypeFound,
}
