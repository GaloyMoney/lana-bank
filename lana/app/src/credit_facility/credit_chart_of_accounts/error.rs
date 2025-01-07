use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditChartOfAccountsError {
    #[error("CreditChartOfAccountsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditChartOfAccountsError - CoreChartOfAccountError: {0}")]
    CoreChartOfAccountError(#[from] chart_of_accounts::error::CoreChartOfAccountError),
}
