use thiserror::Error;

use crate::path::{AccountIdx, ChartCategory};

#[derive(Error, Debug)]
pub enum ChartPathError {
    #[error("ChartError - ParseIntError: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("ChartError - InvalidCategoryForNewControlAccount")]
    InvalidCategoryForNewControlAccount,
    #[error("ChartError - InvalidControlAccountPathForNewControlSubAccount")]
    InvalidControlAccountPathForNewControlSubAccount,
    #[error("ChartError - InvalidSubControlAccountPathForNewTransactionAccount")]
    InvalidSubControlAccountPathForNewTransactionAccount,
    #[error("ChartError - ControlIndexOverflowForCategory: Category '{0}'")]
    ControlIndexOverflowForCategory(ChartCategory),
    #[error(
        "ChartError - ControlSubIndexOverflowForControlAccount: Category '{0}' / Control '{1}'"
    )]
    ControlSubIndexOverflowForControlAccount(ChartCategory, AccountIdx),
    #[error("ChartError - TransactionIndexOverflowForControlSubAccount: Category '{0}' / Control '{1}' / Sub-control '{2}'")]
    TransactionIndexOverflowForControlSubAccount(ChartCategory, AccountIdx, AccountIdx),
    #[error("ChartError - InvalidCodeLength: {0}")]
    InvalidCodeLength(String),
    #[error("ChartError - InvalidCategoryNumber: {0}")]
    InvalidCategoryNumber(u32),
    #[error("ChartError - InvalidCodeString: {0}")]
    InvalidCodeString(String),
}
