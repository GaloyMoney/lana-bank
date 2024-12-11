use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartOfAccountsError {
    #[error("ChartOfAccountsError - InvalidChartOfAccountsCodeStr")]
    InvalidChartOfAccountsCodeStr,
}
