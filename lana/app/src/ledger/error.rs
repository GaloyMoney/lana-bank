use thiserror::Error;

use crate::primitives::UsdCents;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("LedgerError - CalaError: {0}")]
    Cala(#[from] super::cala::error::CalaError),
    #[error("CalaError - TryFromIntError: {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error("LedgerError - ConversionError: {0}")]
    ConversionError(#[from] crate::primitives::ConversionError),
    #[error("LedgerError - CoreChartOfAccountError: {0}")]
    CoreChartOfAccountError(#[from] chart_of_accounts::error::CoreChartOfAccountError),
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
    #[error(
        "LedgerError - WithdrawalAmountTooLarge: amount '{0}' is larger than bank balance '{1}'"
    )]
    WithdrawalAmountTooLarge(UsdCents, UsdCents),
    #[error("LedgerError - AuthorizationError: {0}")]
    AuthorizationError(#[from] crate::authorization::error::AuthorizationError),
    #[error("LedgerError - InsufficientBalance: {0} < {1}")]
    InsufficientBalance(UsdCents, UsdCents),
}
