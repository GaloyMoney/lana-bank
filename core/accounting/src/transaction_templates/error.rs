use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionTemplateError {
    #[error("CoreTransactionTemplateError - TxTemplate: {0}")]
    TxTemplate(#[from] cala_ledger::tx_template::error::TxTemplateError),
}
