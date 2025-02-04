use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalanceSheetLedgerError {
    #[error("BalanceSheetLedgerError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("BalanceSheetLedgerError - CalaLedger: {0}")]
    CalaLedger(#[from] cala_ledger::error::LedgerError),
    #[error("BalanceSheetLedgerError - CalaAccountSet: {0}")]
    CalaAccountSet(#[from] cala_ledger::account_set::error::AccountSetError),
    #[error("BalanceSheetLedgerError - CalaBalance: {0}")]
    CalaBalance(#[from] cala_ledger::balance::error::BalanceError),
    #[error("BalanceSheetError - ConversionError: {0}")]
    Statement(#[from] crate::statement::error::StatementError),
    #[error("BalanceSheetLedgerError - NonAccountSetMemberTypeFound")]
    NonAccountSetMemberTypeFound,
    #[error("BalanceSheetLedgerError - MultipleFound: {0}")]
    MultipleFound(String),
    #[error("BalanceSheetLedgerError - NotFound: {0}")]
    NotFound(String),
}
