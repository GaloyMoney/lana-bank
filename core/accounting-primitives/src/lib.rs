use std::{borrow::Cow, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, BalanceId as CalaBalanceId,
    EntryId as CalaEntryId, JournalId as CalaJournalId, TransactionId as CalaTxId,
    TxTemplateId as CalaTxTemplateId,
};

es_entity::entity_id! {
    ChartId;
    ChartId => CalaAccountSetId,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityType(Cow<'static, str>);
impl EntityType {
    pub const fn new(entity_type: &'static str) -> Self {
        Self(Cow::Borrowed(entity_type))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRef {
    pub entity_type: EntityType,
    pub entity_id: uuid::Uuid,
}

impl EntityRef {
    pub fn new(entity_type: EntityType, id: impl Into<uuid::Uuid>) -> Self {
        Self {
            entity_type,
            entity_id: id.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AccountNameParseError {
    #[error("empty")]
    Empty,
    #[error("starts-with-digit")]
    StartsWithDigit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AccountName {
    name: String,
}

impl Display for AccountName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromStr for AccountName {
    type Err = AccountNameParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(AccountNameParseError::Empty);
        }
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_ascii_digit() {
                return Err(AccountNameParseError::StartsWithDigit);
            }
        } else {
            return Err(AccountNameParseError::Empty);
        }
        Ok(AccountName {
            name: trimmed.to_string(),
        })
    }
}

#[derive(Error, Debug)]
pub enum AccountCodeSectionParseError {
    #[error("empty")]
    Empty,
    #[error("non-digit")]
    NonDigit,
}

#[derive(Error, Debug)]
pub enum AccountCodeParseError {
    #[error("AccountCodeParseError - Empty")]
    Empty,
    #[error("AccountCodeParseError - AccountCodeSectionParseError: {0}")]
    AccountCodeSectionParseError(#[from] AccountCodeSectionParseError),
    #[error("AccountCodeParseError - InvalidParent")]
    InvalidParent,
}

impl ErrorSeverity for AccountCodeParseError {
    fn severity(&self) -> Level {
        match self {
            Self::Empty => Level::WARN,
            Self::AccountCodeSectionParseError(_) => Level::WARN,
            Self::InvalidParent => Level::WARN,
        }
    }
}

#[derive(Error, Debug)]
pub enum AccountCodeError {
    #[error("AccountCodeError - InvalidParent")]
    InvalidParent,
}

impl ErrorSeverity for AccountCodeError {
    fn severity(&self) -> Level {
        match self {
            Self::InvalidParent => Level::WARN,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AccountCodeSection {
    code: String,
}

impl FromStr for AccountCodeSection {
    type Err = AccountCodeSectionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(AccountCodeSectionParseError::Empty);
        }

        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(AccountCodeSectionParseError::NonDigit);
        }

        Ok(AccountCodeSection {
            code: s.to_string(),
        })
    }
}
impl Display for AccountCodeSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AccountCode {
    sections: Vec<AccountCodeSection>,
}

impl From<AccountCode> for Vec<AccountCodeSection> {
    fn from(code: AccountCode) -> Self {
        code.sections
    }
}

impl From<&AccountCode> for Vec<AccountCodeSection> {
    fn from(code: &AccountCode) -> Self {
        code.sections.clone()
    }
}

impl AccountCode {
    pub fn new(section: Vec<AccountCodeSection>) -> Self {
        AccountCode { sections: section }
    }

    pub fn account_set_external_id(&self, chart_id: ChartId) -> String {
        format!("{chart_id}.{self}")
    }

    pub fn manual_account_external_id(&self, chart_id: ChartId) -> String {
        format!("{chart_id}.{self}.manual")
    }

    pub fn len_sections(&self) -> usize {
        self.sections.len()
    }

    pub fn chart_level(&self) -> usize {
        self.len_sections() - 1
    }

    pub fn is_top_level_chart_code(&self) -> bool {
        self.sections.len() == 1 && self.sections.first().is_some_and(|s| s.code.len() == 1)
    }

    pub fn section(&self, idx: usize) -> Option<&AccountCodeSection> {
        self.sections.get(idx)
    }

    pub fn is_equivalent_to_str(&self, code: &str) -> bool {
        let mut position = 0;

        for section in &self.sections {
            let section_len = section.code.len();

            if position + section_len > code.len() {
                return false;
            }

            if code[position..position + section_len] != section.code {
                return false;
            }

            position += section_len;
        }

        position == code.len()
    }

    pub fn is_parent_of(&self, child_sections: &[AccountCodeSection]) -> bool {
        let parent_sections = &self.sections;
        if parent_sections.is_empty() || child_sections.is_empty() {
            return false;
        }

        if parent_sections == child_sections {
            return false;
        }

        for (i, parent_section) in parent_sections.iter().enumerate() {
            if i >= child_sections.len() {
                return false;
            }

            let child_section = &child_sections[i];
            if !child_section.code.starts_with(&parent_section.code) {
                return false;
            }
            if child_section.code.len() <= parent_section.code.len()
                && child_section.code != parent_section.code
            {
                return false;
            }
        }

        true
    }

    pub fn check_valid_parent(
        &self,
        parent_code: Option<AccountCode>,
    ) -> Result<(), AccountCodeError> {
        let parent_code = if let Some(parent_code) = parent_code {
            parent_code
        } else {
            return Ok(());
        };

        if parent_code.is_parent_of(&self.sections) {
            Ok(())
        } else {
            Err(AccountCodeError::InvalidParent)
        }
    }
}

impl FromStr for AccountCode {
    type Err = AccountCodeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(AccountCodeParseError::Empty);
        }

        let account_code = match s.split_once('.') {
            Some((first, rest)) if uuid::Uuid::parse_str(first).is_ok() => rest,
            _ => s,
        };
        let sections = account_code
            .split('.')
            .map(|part| {
                part.parse::<AccountCodeSection>()
                    .map_err(AccountCodeParseError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AccountCode::new(sections))
    }
}

impl Display for AccountCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sections.is_empty() {
            return Ok(());
        }

        write!(f, "{}", self.sections[0])?;

        for section in &self.sections[1..] {
            write!(f, ".{section}")?;
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum AccountingBaseConfigError {
    #[error("AccountingBaseConfigError - DuplicateAccountCode: {0}")]
    DuplicateAccountCode(String),
    #[error("AccountingBaseConfigError - AccountCodeNotTopLevel: {0}")]
    AccountCodeNotTopLevel(String),
    #[error("AccountingBaseConfigError - RetainedEarningsCodeNotChildOfEquity: {0}")]
    RetainedEarningsCodeNotChildOfEquity(String),
}

impl ErrorSeverity for AccountingBaseConfigError {
    fn severity(&self) -> Level {
        match self {
            Self::DuplicateAccountCode(_) => Level::ERROR,
            Self::AccountCodeNotTopLevel(_) => Level::ERROR,
            Self::RetainedEarningsCodeNotChildOfEquity(_) => Level::ERROR,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AccountingBaseConfig {
    pub assets_code: AccountCode,
    pub liabilities_code: AccountCode,
    pub equity_code: AccountCode,
    pub equity_retained_earnings_gain_code: AccountCode,
    pub equity_retained_earnings_loss_code: AccountCode,
    pub revenue_code: AccountCode,
    pub cost_of_revenue_code: AccountCode,
    pub expenses_code: AccountCode,
}

impl AccountingBaseConfig {
    pub fn try_new(
        assets_code: AccountCode,
        liabilities_code: AccountCode,
        equity_code: AccountCode,
        equity_retained_earnings_gain_code: AccountCode,
        equity_retained_earnings_loss_code: AccountCode,
        revenue_code: AccountCode,
        cost_of_revenue_code: AccountCode,
        expenses_code: AccountCode,
    ) -> Result<Self, AccountingBaseConfigError> {
        let config = Self {
            assets_code,
            liabilities_code,
            equity_code,
            equity_retained_earnings_gain_code,
            equity_retained_earnings_loss_code,
            revenue_code,
            cost_of_revenue_code,
            expenses_code,
        };
        config.validate()?;
        Ok(config)
    }
    fn validate(&self) -> Result<(), AccountingBaseConfigError> {
        let codes = [
            &self.assets_code,
            &self.liabilities_code,
            &self.equity_code,
            &self.revenue_code,
            &self.cost_of_revenue_code,
            &self.expenses_code,
        ];
        if let Some(code) = codes.iter().copied().find(|c| !c.is_top_level_chart_code()) {
            return Err(AccountingBaseConfigError::AccountCodeNotTopLevel(
                code.to_string(),
            ));
        }

        let mut seen = std::collections::HashSet::with_capacity(codes.len());
        if let Some(code) = codes.iter().copied().find(|c| !seen.insert(*c)) {
            return Err(AccountingBaseConfigError::DuplicateAccountCode(
                code.to_string(),
            ));
        }

        if !self
            .equity_code
            .is_parent_of(&self.equity_retained_earnings_gain_code.sections)
            || !self
                .equity_code
                .is_parent_of(&self.equity_retained_earnings_loss_code.sections)
        {
            return Err(
                AccountingBaseConfigError::RetainedEarningsCodeNotChildOfEquity(
                    self.equity_code.to_string(),
                ),
            );
        }
        Ok(())
    }

    pub fn is_off_balance_sheet_account_set_or_account(&self, code: &AccountCode) -> bool {
        let on_balance_sheet = [
            &self.assets_code,
            &self.liabilities_code,
            &self.equity_code,
            &self.revenue_code,
            &self.cost_of_revenue_code,
            &self.expenses_code,
        ];

        !on_balance_sheet
            .iter()
            .any(|category| *category == code || category.is_parent_of(&code.sections))
    }

    pub fn is_assets_account_set_or_account(&self, code: &AccountCode) -> bool {
        self.assets_code == *code || self.assets_code.is_parent_of(&code.sections)
    }

    pub fn is_liabilities_account_set_or_account(&self, code: &AccountCode) -> bool {
        self.liabilities_code == *code || self.liabilities_code.is_parent_of(&code.sections)
    }

    pub fn is_equity_account_set_or_account(&self, code: &AccountCode) -> bool {
        self.equity_code == *code || self.equity_code.is_parent_of(&code.sections)
    }

    pub fn is_revenue_account_set_or_account(&self, code: &AccountCode) -> bool {
        self.revenue_code == *code || self.revenue_code.is_parent_of(&code.sections)
    }

    pub fn is_cost_of_revenue_account_set_or_account(&self, code: &AccountCode) -> bool {
        self.cost_of_revenue_code == *code || self.cost_of_revenue_code.is_parent_of(&code.sections)
    }

    pub fn is_expenses_account_set_or_account(&self, code: &AccountCode) -> bool {
        self.expenses_code == *code || self.expenses_code.is_parent_of(&code.sections)
    }

    pub fn is_account_in_category(&self, code: &AccountCode, category: AccountCategory) -> bool {
        match category {
            AccountCategory::OffBalanceSheet => {
                self.is_off_balance_sheet_account_set_or_account(code)
            }
            AccountCategory::Asset => self.is_assets_account_set_or_account(code),
            AccountCategory::Liability => self.is_liabilities_account_set_or_account(code),
            AccountCategory::Equity => self.is_equity_account_set_or_account(code),
            AccountCategory::Revenue => self.is_revenue_account_set_or_account(code),
            AccountCategory::CostOfRevenue => self.is_cost_of_revenue_account_set_or_account(code),
            AccountCategory::Expenses => self.is_expenses_account_set_or_account(code),
        }
    }

    pub fn code_for_category(&self, category: AccountCategory) -> Option<&AccountCode> {
        match category {
            AccountCategory::OffBalanceSheet => None,
            AccountCategory::Asset => Some(&self.assets_code),
            AccountCategory::Liability => Some(&self.liabilities_code),
            AccountCategory::Equity => Some(&self.equity_code),
            AccountCategory::Revenue => Some(&self.revenue_code),
            AccountCategory::CostOfRevenue => Some(&self.cost_of_revenue_code),
            AccountCategory::Expenses => Some(&self.expenses_code),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum AccountCategory {
    OffBalanceSheet,
    Asset,
    Liability,
    Equity,
    Revenue,
    CostOfRevenue,
    Expenses,
}

/// Error for chart lookup failures during account set resolution.
#[derive(Error, Debug)]
pub enum ChartLookupError {
    #[error("InvalidAccountCategory: code {code} is not in category {category:?}")]
    InvalidAccountCategory {
        code: AccountCode,
        category: AccountCategory,
    },
}

impl ErrorSeverity for ChartLookupError {
    fn severity(&self) -> Level {
        match self {
            Self::InvalidAccountCategory { .. } => Level::ERROR,
        }
    }
}
