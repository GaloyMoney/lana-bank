use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreditChartOfAccountsError {
    #[error("CreditChartOfAccountsError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CreditChartOfAccountsError - CoreChartOfAccountsError: {0}")]
    CoreChartOfAccountsError(#[from] chart_of_accounts::error::CoreChartOfAccountsError),
}
