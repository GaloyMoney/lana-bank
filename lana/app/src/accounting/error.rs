use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountingError {
    #[error("AccountingError - CoreChartOfAccountError: {0}")]
    CoreChartOfAccountError(#[from] chart_of_accounts::error::CoreChartOfAccountError),
}
