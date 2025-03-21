use cala_ledger::DebitOrCredit;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use authz::AllOrOne;

pub use cala_ledger::primitives::{
    AccountId as LedgerAccountId, AccountSetId as LedgerAccountSetId, JournalId as LedgerJournalId,
};

es_entity::entity_id! {
    ChartId,
}

pub struct AccountDetails {
    pub id: LedgerAccountSetId,
    pub name: AccountName,
    pub code: AccountCode,
}

impl From<&(AccountSpec, LedgerAccountSetId)> for AccountDetails {
    fn from((spec, id): &(AccountSpec, LedgerAccountSetId)) -> Self {
        AccountDetails {
            id: *id,
            name: spec.name.clone(),
            code: spec.code.clone(),
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
pub struct AccountName {
    name: String,
}

impl std::fmt::Display for AccountName {
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
        if trimmed.chars().next().unwrap().is_ascii_digit() {
            return Err(AccountNameParseError::StartsWithDigit);
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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
impl std::fmt::Display for AccountCodeSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AccountCode {
    sections: Vec<AccountCodeSection>,
}
impl AccountCode {
    pub fn new(section: Vec<AccountCodeSection>) -> Self {
        AccountCode { sections: section }
    }

    pub fn len_sections(&self) -> usize {
        self.sections.len()
    }

    pub fn chart_level(&self) -> usize {
        self.len_sections() - 1
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

    pub fn is_parent(&self, sections: &[AccountCodeSection]) -> bool {
        if self.sections.is_empty() {
            return false;
        }
        if sections.is_empty() {
            return false;
        }

        for (i, parent_section) in self.sections.iter().enumerate() {
            if i >= sections.len() {
                return false;
            }

            if !sections[i].code.starts_with(&parent_section.code) {
                return false;
            }
            if sections[i].code.len() <= parent_section.code.len()
                && sections[i].code != parent_section.code
            {
                return false;
            }
        }

        true
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

impl std::fmt::Display for AccountCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sections.is_empty() {
            return Ok(());
        }

        write!(f, "{}", self.sections[0])?;

        for section in &self.sections[1..] {
            write!(f, ".{}", section)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSpec {
    pub parent: Option<AccountCode>,
    pub code: AccountCode,
    pub name: AccountName,
    pub normal_balance_type: DebitOrCredit,
}

impl AccountSpec {
    pub(super) fn new(
        parent: Option<AccountCode>,
        sections: Vec<AccountCodeSection>,
        name: AccountName,
        normal_balance_type: DebitOrCredit,
    ) -> Self {
        let code = AccountCode { sections };
        AccountSpec {
            parent,
            code,
            name,
            normal_balance_type,
        }
    }

    pub(super) fn account_set_external_id(&self, chart_id: ChartId) -> String {
        format!("{}.{}", chart_id, self.code)
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
}

pub type ChartAllOrOne = AllOrOne<ChartId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreChartOfAccountsAction {
    ChartAction(ChartAction),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreChartOfAccountsObject {
    Chart(ChartAllOrOne),
}

impl CoreChartOfAccountsObject {
    pub fn chart(id: ChartId) -> Self {
        CoreChartOfAccountsObject::Chart(AllOrOne::ById(id))
    }

    pub fn all_charts() -> Self {
        CoreChartOfAccountsObject::Chart(AllOrOne::All)
    }
}

impl Display for CoreChartOfAccountsObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreChartOfAccountsObjectDiscriminants::from(self);
        use CoreChartOfAccountsObject::*;
        match self {
            Chart(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CoreChartOfAccountsObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreChartOfAccountsObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Chart => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreChartObject")?;
                CoreChartOfAccountsObject::Chart(obj_ref)
            }
        };
        Ok(res)
    }
}

impl CoreChartOfAccountsAction {
    pub const CHART_CREATE: Self = CoreChartOfAccountsAction::ChartAction(ChartAction::Create);
    pub const CHART_LIST: Self = CoreChartOfAccountsAction::ChartAction(ChartAction::List);
    pub const CHART_ACCOUNT_DETAILS_READ: Self =
        CoreChartOfAccountsAction::ChartAction(ChartAction::AccountDetailsRead);
    pub const CHART_IMPORT_ACCOUNTS: Self =
        CoreChartOfAccountsAction::ChartAction(ChartAction::ImportAccounts);
}

impl Display for CoreChartOfAccountsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreChartOfAccountsActionDiscriminants::from(self))?;
        use CoreChartOfAccountsAction::*;
        match self {
            ChartAction(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreChartOfAccountsAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        let res = match entity.parse()? {
            CoreChartOfAccountsActionDiscriminants::ChartAction => {
                CoreChartOfAccountsAction::from(action.parse::<ChartAction>()?)
            }
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ChartAction {
    Create,
    List,
    AccountDetailsRead,
    ImportAccounts,
    CreateControlAccount,
    FindControlAccount,
    CreateControlSubAccount,
    FindControlSubAccount,
}

impl From<ChartAction> for CoreChartOfAccountsAction {
    fn from(action: ChartAction) -> Self {
        CoreChartOfAccountsAction::ChartAction(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_parent_same_level() {
        let parent = "1".parse::<AccountCodeSection>().unwrap();
        let child = "11".parse::<AccountCodeSection>().unwrap();
        let account_code = AccountCode::new(vec![parent]);
        assert!(account_code.is_parent(&[child]));
    }

    #[test]
    fn is_parent_next_level() {
        let parent = "11".parse::<AccountCodeSection>().unwrap();
        let child = "0201".parse::<AccountCodeSection>().unwrap();
        let account_code = AccountCode::new(vec![parent.clone()]);
        assert!(account_code.is_parent(&[parent, child]));
    }

    #[test]
    fn is_parent_next_level_with_sub() {
        let parent = "11".parse::<AccountCodeSection>().unwrap();
        let sub = "01".parse::<AccountCodeSection>().unwrap();
        let child = "0201".parse::<AccountCodeSection>().unwrap();
        let account_code = AccountCode::new(vec![parent.clone(), sub.clone()]);
        assert!(account_code.is_parent(&[parent, sub, child]));
    }

    #[test]
    fn chart_level() {
        let parent = "11".parse::<AccountCodeSection>().unwrap();
        let sub = "01".parse::<AccountCodeSection>().unwrap();
        let child = "0201".parse::<AccountCodeSection>().unwrap();

        let account_code = AccountCode::new(vec![parent.clone()]);
        assert_eq!(account_code.chart_level(), 0);

        let account_code = AccountCode::new(vec![parent.clone(), sub.clone()]);
        assert_eq!(account_code.chart_level(), 1);

        let account_code = AccountCode::new(vec![parent, sub, child]);
        assert_eq!(account_code.chart_level(), 2);
    }

    #[test]
    fn is_equivalent_to_str() {
        let parent = "11".parse::<AccountCodeSection>().unwrap();
        let sub = "01".parse::<AccountCodeSection>().unwrap();
        let child = "0201".parse::<AccountCodeSection>().unwrap();

        let account_code = AccountCode::new(vec![parent, sub, child]);
        assert!(account_code.is_equivalent_to_str("11010201"));
        assert!(!account_code.is_equivalent_to_str("110102010"));
    }
}
