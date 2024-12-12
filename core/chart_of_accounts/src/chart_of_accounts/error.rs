use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartOfAccountError {
    #[error("ChartOfAccountError - InvalidChartOfAccountCodeStr")]
    InvalidChartOfAccountCodeStr,
}
