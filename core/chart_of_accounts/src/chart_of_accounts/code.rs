use serde::{Deserialize, Serialize};

use std::fmt;

use super::error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash, Deserialize)]
pub struct AccountIdx(u64);
impl fmt::Display for AccountIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl AccountIdx {
    pub const FIRST: Self = Self(1);

    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ChartOfAccountCodeStr([u8; 8]);

impl ChartOfAccountCodeStr {
    fn new(s: &str) -> Result<Self, ChartOfAccountError> {
        if s.len() != 8 || !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(ChartOfAccountError::InvalidChartOfAccountCodeStr);
        }

        let mut code = [0u8; 8];
        for (i, c) in s.bytes().enumerate() {
            code[i] = c;
        }
        Ok(ChartOfAccountCodeStr(code))
    }
}

impl std::fmt::Display for ChartOfAccountCodeStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = std::str::from_utf8(&self.0).unwrap();
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartOfAccountCategoryCode {
    Assets = 1,
    Liabilities = 2,
    Equity = 3,
    Revenues = 4,
    Expenses = 5,
}

impl ChartOfAccountCategoryCode {
    fn index(&self) -> AccountIdx {
        AccountIdx(*self as u64)
    }

    fn code(&self) -> ChartOfAccountCodeStr {
        ChartOfAccountCodeStr::new(&format!("{:01}0000000", *self as u64))
            .expect("Invalid category code string")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartOfAccountControlAccountCode {
    pub category: ChartOfAccountCategoryCode,
    index: AccountIdx,
}

impl ChartOfAccountControlAccountCode {
    pub const fn first(category: ChartOfAccountCategoryCode) -> Self {
        Self {
            category,
            index: AccountIdx::FIRST,
        }
    }

    pub const fn next(&self) -> Self {
        Self {
            category: self.category,
            index: self.index.next(),
        }
    }

    fn code(&self) -> ChartOfAccountCodeStr {
        ChartOfAccountCodeStr::new(&format!("{}{:02}000000", self.category.index(), self.index))
            .expect("Invalid control account code string")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartOfAccountControlSubAccountCode {
    pub control_account: ChartOfAccountControlAccountCode,
    index: AccountIdx,
}

impl ChartOfAccountControlSubAccountCode {
    pub const fn first(control_account: ChartOfAccountControlAccountCode) -> Self {
        Self {
            control_account,
            index: AccountIdx::FIRST,
        }
    }

    pub const fn next(&self) -> Self {
        Self {
            control_account: self.control_account,
            index: self.index.next(),
        }
    }

    fn code(&self) -> ChartOfAccountCodeStr {
        ChartOfAccountCodeStr::new(&format!(
            "{}{:02}{:02}000",
            self.control_account.category.index(),
            self.control_account.index,
            self.index
        ))
        .expect("Invalid control sub-account code string")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartOfAccountTransactionAccountCode {
    pub control_sub_account: ChartOfAccountControlSubAccountCode,
    index: AccountIdx,
}

impl ChartOfAccountTransactionAccountCode {
    pub const fn first(control_sub_account: ChartOfAccountControlSubAccountCode) -> Self {
        Self {
            control_sub_account,
            index: AccountIdx::FIRST,
        }
    }

    pub const fn next(&self) -> Self {
        Self {
            control_sub_account: self.control_sub_account,
            index: self.index.next(),
        }
    }

    pub fn code(&self) -> ChartOfAccountCodeStr {
        ChartOfAccountCodeStr::new(&format!(
            "{}{:02}{:02}{:03}",
            self.control_sub_account.control_account.category.index(),
            self.control_sub_account.control_account.index,
            self.control_sub_account.index,
            self.index
        ))
        .expect("Invalid transaction account code string")
    }
}
