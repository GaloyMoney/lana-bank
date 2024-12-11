use super::error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChartOfAccountsCode {
    Category(ChartOfAccountsCategory),
    ControlAccount(ChartOfAccountsControlAccount),
    ControlSubAccount(ChartOfAccountsControlSubAccount),
    Account(ChartOfAccountsAccount),
}

impl ChartOfAccountsCode {
    fn code(&self) -> ChartOfAccountsCodeStr {
        match self {
            ChartOfAccountsCode::Category(category) => category.code(),
            ChartOfAccountsCode::ControlAccount(control) => control.code(),
            ChartOfAccountsCode::ControlSubAccount(sub) => sub.code(),
            ChartOfAccountsCode::Account(account) => account.code(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ChartOfAccountsCodeStr([u8; 8]);

impl ChartOfAccountsCodeStr {
    fn new(s: &str) -> Result<Self, ChartOfAccountsError> {
        if s.len() != 8 || !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(ChartOfAccountsError::InvalidChartOfAccountsCodeStr);
        }

        let mut code = [0u8; 8];
        for (i, c) in s.bytes().enumerate() {
            code[i] = c;
        }
        Ok(ChartOfAccountsCodeStr(code))
    }
}

impl std::fmt::Display for ChartOfAccountsCodeStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = std::str::from_utf8(&self.0).unwrap();
        write!(f, "{}", s)
    }
}

enum ChartOfAccountsCategory {
    ASSETS = 1,
    LIABILITIES = 2,
    EQUITY = 3,
    REVENUES = 4,
    EXPENSES = 5,
}

impl ChartOfAccountsCategory {
    fn as_index() -> u8 {
        Self as u8
    }

    fn code(&self) -> ChartOfAccountsCodeStr {
        ChartOfAccountsCodeStr::new(&format!("{:01}0000000", *self as u8))
            .expect("Invalid category code string")
    }
}

struct ChartOfAccountsControlAccount {
    category: ChartOfAccountsCategory,
    index: u8,
}

impl ChartOfAccountsControlAccount {
    fn new(category: ChartOfAccountsCategory, index: u8) -> Self {
        Self { category, index }
    }

    fn code(&self) -> String {
        ChartOfAccountsCodeStr::new(&format!("{}{:02}000000", self.category.index(), self.index))
            .expect("Invalid control account code string")
    }
}

struct ChartOfAccountsControlSubAccount {
    control_account: ChartOfAccountsControlAccount,
    index: u8,
}

impl ChartOfAccountsControlSubAccount {
    fn new(control_account: ChartOfAccountsControlAccount, index: u8) -> Self {
        Self {
            control_account,
            index,
        }
    }

    fn code(&self) -> String {
        ChartOfAccountsCodeStr::new(&format!(
            "{}{:02}{:02}000",
            self.control_account.category.index(),
            self.control_account.index,
            self.index
        ))
        .expect("Invalid control sub-account code string")
    }
}

struct ChartOfAccountsAccount {
    control_sub_account: ChartOfAccountsControlSubAccount,
    id: LedgerAccountId,
    index: u8,
}

impl ChartOfAccountsAccount {
    fn new(control_sub_account: ChartOfAccountsControlSubAccount, index: u8) -> Self {
        Self {
            control_sub_account,
            index,
            id: LedgerAccountId::new(),
        }
    }

    fn code(&self) -> String {
        ChartOfAccountsCodeStr::new(&format!(
            "{}{:02}{:02}{:03}",
            self.control_sub_account.control_account.index(),
            self.control_sub_account.control_account.index,
            self.control_sub_account.index,
            self.index
        ))
        .expect("Invalid account code string")
    }
}
