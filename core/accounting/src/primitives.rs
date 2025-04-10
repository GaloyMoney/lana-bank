use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use authz::AllOrOne;

pub use cala_ledger::{
    Currency as CalaCurrency, DebitOrCredit,
    account::Account as CalaAccount,
    account_set::AccountSet as CalaAccountSet,
    balance::{AccountBalance as CalaAccountBalance, BalanceRange as CalaBalanceRange},
    primitives::{
        AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, EntryId as CalaEntryId,
        JournalId as CalaJournalId, TransactionId as CalaTxId,
    },
};

pub use core_money::{Satoshis, UsdCents};

es_entity::entity_id! {
    ChartId,
    ManualTransactionId,
    LedgerAccountId;

    LedgerAccountId => CalaAccountId,
    LedgerAccountId => CalaAccountSetId
}

impl From<cala_ledger::account_set::AccountSetMemberId> for LedgerAccountId {
    fn from(value: cala_ledger::account_set::AccountSetMemberId) -> Self {
        match value {
            cala_ledger::account_set::AccountSetMemberId::Account(id) => id.into(),
            cala_ledger::account_set::AccountSetMemberId::AccountSet(id) => id.into(),
        }
    }
}

pub type LedgerTransactionId = CalaTxId;

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

    pub(super) fn account_set_external_id(&self, chart_id: ChartId) -> String {
        format!("{}.{}", chart_id, self)
    }

    pub(super) fn manual_account_external_id(&self, chart_id: ChartId) -> String {
        format!("{}.{}.manual", chart_id, self)
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

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
}

pub type ChartAllOrOne = AllOrOne<ChartId>;
pub type JournalAllOrOne = AllOrOne<CalaJournalId>;
pub type LedgerAccountAllOrOne = AllOrOne<LedgerAccountId>;
pub type LedgerTransactionAllOrOne = AllOrOne<CalaTxId>;
pub type ManualTransactionAllOrOne = AllOrOne<ManualTransactionId>;
pub type ProfitAndLossAllOrOne = AllOrOne<LedgerAccountId>;
pub type ProfitAndLossConfigurationAllOrOne = AllOrOne<LedgerAccountId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccountingAction {
    ChartAction(ChartAction),
    JournalAction(JournalAction),
    LedgerAccountAction(LedgerAccountAction),
    LedgerTransactionAction(LedgerTransactionAction),
    ManualTransactionAction(ManualTransactionAction),
    ProfitAndLossAction(ProfitAndLossAction),
    ProfitAndLossConfigurationAction(ProfitAndLossConfigurationAction),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccountingObject {
    Chart(ChartAllOrOne),
    Journal(JournalAllOrOne),
    LedgerAccount(LedgerAccountAllOrOne),
    LedgerTransaction(LedgerTransactionAllOrOne),
    ManualTransaction(ManualTransactionAllOrOne),
    ProfitAndLoss(ProfitAndLossAllOrOne),
    ProfitAndLossConfigration(ProfitAndLossConfigurationAllOrOne),
}

impl CoreAccountingObject {
    pub fn chart(id: ChartId) -> Self {
        CoreAccountingObject::Chart(AllOrOne::ById(id))
    }

    pub fn all_charts() -> Self {
        CoreAccountingObject::Chart(AllOrOne::All)
    }

    pub fn all_journals() -> Self {
        CoreAccountingObject::Journal(AllOrOne::All)
    }

    pub fn journal(id: CalaJournalId) -> Self {
        CoreAccountingObject::Journal(AllOrOne::ById(id))
    }

    pub fn all_ledger_accounts() -> Self {
        CoreAccountingObject::LedgerAccount(AllOrOne::All)
    }

    pub fn ledger_account(id: LedgerAccountId) -> Self {
        CoreAccountingObject::LedgerAccount(AllOrOne::ById(id))
    }

    pub fn all_ledger_transactions() -> Self {
        CoreAccountingObject::LedgerTransaction(AllOrOne::All)
    }

    pub fn ledger_transaction(id: LedgerTransactionId) -> Self {
        CoreAccountingObject::LedgerTransaction(AllOrOne::ById(id))
    }

    pub fn all_manual_transactions() -> Self {
        CoreAccountingObject::ManualTransaction(AllOrOne::All)
    }

    pub fn manual_transaction(id: ManualTransactionId) -> Self {
        CoreAccountingObject::ManualTransaction(AllOrOne::ById(id))
    }

    pub fn all_profit_and_loss() -> Self {
        CoreAccountingObject::ProfitAndLoss(AllOrOne::All)
    }

    pub fn profit_and_loss(id: LedgerAccountId) -> Self {
        CoreAccountingObject::ProfitAndLoss(AllOrOne::ById(id))
    }

    pub fn all_profit_and_loss_configuration() -> Self {
        CoreAccountingObject::ProfitAndLossConfigration(AllOrOne::All)
    }
}

impl Display for CoreAccountingObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreAccountingObjectDiscriminants::from(self);
        use CoreAccountingObject::*;
        match self {
            Chart(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Journal(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            LedgerAccount(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            LedgerTransaction(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            ManualTransaction(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            ProfitAndLoss(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            ProfitAndLossConfigration(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CoreAccountingObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreAccountingObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Chart => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreChartObject")?;
                CoreAccountingObject::Chart(obj_ref)
            }
            Journal => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreJournalObject")?;
                CoreAccountingObject::Journal(obj_ref)
            }
            LedgerAccount => {
                let obj_ref = id.parse().map_err(|_| "could not parse LedgerAccount")?;
                CoreAccountingObject::LedgerAccount(obj_ref)
            }
            LedgerTransaction => {
                let obj_ref = id.parse().map_err(|_| "could not parse LedgerAccount")?;
                CoreAccountingObject::LedgerTransaction(obj_ref)
            }
            ManualTransaction => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse ManualTransaction")?;
                CoreAccountingObject::ManualTransaction(obj_ref)
            }
            ProfitAndLoss => {
                let obj_ref = id.parse().map_err(|_| "could not parse ProfitAndLoss")?;
                CoreAccountingObject::ProfitAndLoss(obj_ref)
            }
            ProfitAndLossConfigration => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse ProfitAndLossConfiguration")?;
                CoreAccountingObject::ProfitAndLossConfigration(obj_ref)
            }
        };
        Ok(res)
    }
}

impl CoreAccountingAction {
    pub const CHART_CREATE: Self = CoreAccountingAction::ChartAction(ChartAction::Create);
    pub const CHART_LIST: Self = CoreAccountingAction::ChartAction(ChartAction::List);
    pub const CHART_IMPORT_ACCOUNTS: Self =
        CoreAccountingAction::ChartAction(ChartAction::ImportAccounts);

    pub const JOURNAL_READ_ENTRIES: Self =
        CoreAccountingAction::JournalAction(JournalAction::ReadEntries);

    pub const LEDGER_ACCOUNT_READ: Self =
        CoreAccountingAction::LedgerAccountAction(LedgerAccountAction::Read);
    pub const LEDGER_ACCOUNT_LIST: Self =
        CoreAccountingAction::LedgerAccountAction(LedgerAccountAction::List);
    pub const LEDGER_ACCOUNT_READ_HISTORY: Self =
        CoreAccountingAction::LedgerAccountAction(LedgerAccountAction::ReadHistory);

    pub const LEDGER_TRANSACTION_READ: Self =
        CoreAccountingAction::LedgerTransactionAction(LedgerTransactionAction::Read);

    pub const MANUAL_TRANSACTION_READ: Self =
        CoreAccountingAction::ManualTransactionAction(ManualTransactionAction::Read);
    pub const MANUAL_TRANSACTION_CREATE: Self =
        CoreAccountingAction::ManualTransactionAction(ManualTransactionAction::Create);
    pub const MANUAL_TRANSACTION_LIST: Self =
        CoreAccountingAction::ManualTransactionAction(ManualTransactionAction::List);
    pub const PROFIT_AND_LOSS_READ: Self =
        CoreAccountingAction::ProfitAndLossAction(ProfitAndLossAction::Read);
    pub const PROFIT_AND_LOSS_CREATE: Self =
        CoreAccountingAction::ProfitAndLossAction(ProfitAndLossAction::Create);
    pub const PROFIT_AND_LOSS_UPDATE: Self =
        CoreAccountingAction::ProfitAndLossAction(ProfitAndLossAction::Update);
    pub const PROFIT_AND_LOSS_CONFIGURATION_READ: Self =
        CoreAccountingAction::ProfitAndLossConfigurationAction(
            ProfitAndLossConfigurationAction::Read,
        );
    pub const PROFIT_AND_LOSS_CONFIGURATION_UPDATE: Self =
        CoreAccountingAction::ProfitAndLossConfigurationAction(
            ProfitAndLossConfigurationAction::Update,
        );
}

impl Display for CoreAccountingAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreAccountingActionDiscriminants::from(self))?;
        use CoreAccountingAction::*;
        match self {
            ChartAction(action) => action.fmt(f),
            JournalAction(action) => action.fmt(f),
            LedgerAccountAction(action) => action.fmt(f),
            LedgerTransactionAction(action) => action.fmt(f),
            ManualTransactionAction(action) => action.fmt(f),
            ProfitAndLossAction(action) => action.fmt(f),
            ProfitAndLossConfigurationAction(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreAccountingAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        let res = match entity.parse()? {
            CoreAccountingActionDiscriminants::ChartAction => {
                CoreAccountingAction::from(action.parse::<ChartAction>()?)
            }
            CoreAccountingActionDiscriminants::JournalAction => {
                CoreAccountingAction::from(action.parse::<JournalAction>()?)
            }
            CoreAccountingActionDiscriminants::LedgerAccountAction => {
                CoreAccountingAction::from(action.parse::<LedgerAccountAction>()?)
            }
            CoreAccountingActionDiscriminants::LedgerTransactionAction => {
                CoreAccountingAction::from(action.parse::<LedgerTransactionAction>()?)
            }
            CoreAccountingActionDiscriminants::ManualTransactionAction => {
                CoreAccountingAction::from(action.parse::<ManualTransactionAction>()?)
            }
            CoreAccountingActionDiscriminants::ProfitAndLossAction => {
                CoreAccountingAction::from(action.parse::<ProfitAndLossAction>()?)
            }
            CoreAccountingActionDiscriminants::ProfitAndLossConfigurationAction => {
                CoreAccountingAction::from(action.parse::<ProfitAndLossConfigurationAction>()?)
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
    ImportAccounts,
}

impl From<ChartAction> for CoreAccountingAction {
    fn from(action: ChartAction) -> Self {
        CoreAccountingAction::ChartAction(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum LedgerTransactionAction {
    Read,
    List,
    ReadHistory,
}

impl From<LedgerTransactionAction> for CoreAccountingAction {
    fn from(action: LedgerTransactionAction) -> Self {
        CoreAccountingAction::LedgerTransactionAction(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum LedgerAccountAction {
    Read,
    List,
    ReadHistory,
}

impl From<LedgerAccountAction> for CoreAccountingAction {
    fn from(action: LedgerAccountAction) -> Self {
        CoreAccountingAction::LedgerAccountAction(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum JournalAction {
    ReadEntries,
}

impl From<JournalAction> for CoreAccountingAction {
    fn from(action: JournalAction) -> Self {
        CoreAccountingAction::JournalAction(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ManualTransactionAction {
    Read,
    Create,
    List,
}

impl From<ManualTransactionAction> for CoreAccountingAction {
    fn from(action: ManualTransactionAction) -> Self {
        CoreAccountingAction::ManualTransactionAction(action)
    }
}

#[derive(Debug, Clone)]
pub struct BalanceRange {
    pub start: Option<CalaAccountBalance>,
    pub end: Option<CalaAccountBalance>,
    pub diff: Option<CalaAccountBalance>,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ProfitAndLossAction {
    Read,
    Create,
    Update,
}

impl From<ProfitAndLossAction> for CoreAccountingAction {
    fn from(action: ProfitAndLossAction) -> Self {
        CoreAccountingAction::ProfitAndLossAction(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
pub enum ProfitAndLossConfigurationAction {
    Read,
    Update,
}

impl From<ProfitAndLossConfigurationAction> for CoreAccountingAction {
    fn from(action: ProfitAndLossConfigurationAction) -> Self {
        CoreAccountingAction::ProfitAndLossConfigurationAction(action)
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
