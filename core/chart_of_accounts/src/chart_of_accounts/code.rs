use super::error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChartOfAccountCategoryCode {
    Assets = 1,
    Liabilities = 2,
    Equity = 3,
    Revenues = 4,
    Expenses = 5,
}

impl ChartOfAccountCategoryCode {
    fn index(&self) -> u8 {
        *self as u8
    }

    fn code(&self) -> ChartOfAccountCodeStr {
        ChartOfAccountCodeStr::new(&format!("{:01}0000000", *self as u8))
            .expect("Invalid category code string")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ChartOfAccountControlAccountCode {
    category: ChartOfAccountCategoryCode,
    index: u8,
}

impl ChartOfAccountControlAccountCode {
    fn new(category: ChartOfAccountCategoryCode, index: u8) -> Self {
        Self { category, index }
    }

    fn code(&self) -> ChartOfAccountCodeStr {
        ChartOfAccountCodeStr::new(&format!("{}{:02}000000", self.category.index(), self.index))
            .expect("Invalid control account code string")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ChartOfAccountControlSubAccountCode {
    control_account: ChartOfAccountControlAccountCode,
    index: u8,
}

impl ChartOfAccountControlSubAccountCode {
    fn new(control_account: ChartOfAccountControlAccountCode, index: u8) -> Self {
        Self {
            control_account,
            index,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ChartOfAccountTransactionAccountCode {
    control_sub_account: ChartOfAccountControlSubAccountCode,
    index: u8,
}

impl ChartOfAccountTransactionAccountCode {
    fn new(control_sub_account: ChartOfAccountControlSubAccountCode, index: u8) -> Self {
        Self {
            control_sub_account,
            index,
        }
    }

    fn code(&self) -> ChartOfAccountCodeStr {
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
