use thiserror::Error;

use crate::code::{AccountIdx, ChartOfAccountCategoryCode};

#[derive(Error, Debug)]
pub enum ChartOfAccountCodeError {
    #[error("ChartOfAccountError - ParseIntError: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("ChartOfAccountError - InvalidCategoryCodeForNewControlAccount")]
    InvalidCategoryCodeForNewControlAccount,
    #[error("ChartOfAccountError - InvalidControlAccountCodeForNewControlSubAccount")]
    InvalidControlAccountCodeForNewControlSubAccount,
    #[error("ChartOfAccountError - InvalidSubControlAccountCodeForNewTransactionAccount")]
    InvalidSubControlAccountCodeForNewTransactionAccount,
    #[error("ChartOfAccountError - ControlIndexOverflowForCategory: Category '{0}'")]
    ControlIndexOverflowForCategory(ChartOfAccountCategoryCode),
    #[error("ChartOfAccountError - ControlSubIndexOverflowForControlAccount: Category '{0}' / Control '{1}'")]
    ControlSubIndexOverflowForControlAccount(ChartOfAccountCategoryCode, AccountIdx),
    #[error("ChartOfAccountError - TransactionIndexOverflowForControlSubAccount: Category '{0}' / Control '{1}' / Sub-control '{2}'")]
    TransactionIndexOverflowForControlSubAccount(
        ChartOfAccountCategoryCode,
        AccountIdx,
        AccountIdx,
    ),
    #[error("ChartOfAccountError - InvalidCodeLength: {0}")]
    InvalidCodeLength(String),
    #[error("ChartOfAccountError - InvalidCategoryNumber: {0}")]
    InvalidCategoryNumber(u32),
    #[error("ChartOfAccountError - InvalidCodeString: {0}")]
    InvalidCodeString(String),
}
