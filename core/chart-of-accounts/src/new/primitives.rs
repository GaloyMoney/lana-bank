use std::{fmt::Display, str::FromStr};

use authz::AllOrOne;

pub use cala_ledger::primitives::JournalId as LedgerJournalId;

es_entity::entity_id! {
    ChartId,
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountCategoryParseError {
    #[error("empty")]
    Empty,
    #[error("starts-with-digit")]
    StartsWithDigit,
}

#[derive(Debug)]
pub struct AccountCategory {
    name: String,
}

impl FromStr for AccountCategory {
    type Err = AccountCategoryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(AccountCategoryParseError::Empty);
        }
        if trimmed.chars().next().unwrap().is_ascii_digit() {
            return Err(AccountCategoryParseError::StartsWithDigit);
        }
        Ok(AccountCategory {
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct AccountCode {
    section: Vec<AccountCodeSection>,
}
impl AccountCode {
    pub fn new(section: Vec<AccountCodeSection>) -> Self {
        AccountCode { section }
    }

    pub fn len_sections(&self) -> usize {
        self.section.len()
    }

    pub fn section(&self, idx: usize) -> Option<&AccountCodeSection> {
        self.section.get(idx)
    }

    pub fn is_parent(&self, sections: &[AccountCodeSection]) -> bool {
        if self.section.is_empty() {
            return false;
        }
        if sections.is_empty() {
            return false;
        }

        for (i, parent_section) in self.section.iter().enumerate() {
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

impl std::fmt::Display for AccountCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.section.is_empty() {
            return Ok(());
        }

        write!(f, "{}", self.section[0])?;

        for section in &self.section[1..] {
            write!(f, ".{}", section)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AccountSpec {
    pub parent: Option<AccountCode>,
    pub code: AccountCode,
    pub category: AccountCategory,
}

impl AccountSpec {
    pub(super) fn new(
        parent: Option<AccountCode>,
        sections: Vec<AccountCodeSection>,
        category: AccountCategory,
    ) -> Self {
        let code = AccountCode { section: sections };
        AccountSpec {
            parent,
            code,
            category,
        }
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
}
