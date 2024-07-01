use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("LedgerError - CalaError: {0}")]
    Cala(#[from] super::cala::error::CalaError),
    #[error("LedgerError - CouldNotAssertAccountExists")]
    CouldNotAssertAccountExists,
    #[error("LedgerError - CouldNotAssertAccountSetExists")]
    CouldNotAssertAccountSetExists,
    #[error("LedgerError - CouldNotAssertAccountIsMemberOfAccountSet")]
    CouldNotAssertAccountIsMemberOfAccountSet,
    #[error("LedgerError - CouldNotAssertTxTemplateExists")]
    CouldNotAssertTxTemplateExists,
    #[error("LedgerError - CouldNotAssertBfxIntegrationExists")]
    CouldNotAssertBfxIntegrationExists,
    #[error("LedgerError - CouldNotInitializeJournal")]
    CouldNotInitializeJournal,
    #[error("LedgerError - AccountNotFound")]
    AccountNotFound,
}
