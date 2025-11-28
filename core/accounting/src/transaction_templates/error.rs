use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum TransactionTemplateError {
    #[error("CoreTransactionTemplateError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreTransactionTemplateError - TxTemplate: {0}")]
    TxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
}

impl ErrorSeverity for TransactionTemplateError {
    fn severity(&self) -> Level {
        match self {
            Self::AuthorizationError(e) => e.severity(),
            Self::TxTemplate(_) => Level::ERROR,
        }
    }
}
