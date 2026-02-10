use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display, str::FromStr};
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub const ACCOUNTING_TRIAL_BALANCE: audit::SystemActor =
    audit::SystemActor::new("accounting-trial-balance");

pub use cala_ledger::{
    Currency as CalaCurrency, DebitOrCredit,
    account::Account as CalaAccount,
    account_set::AccountSet as CalaAccountSet,
    balance::{AccountBalance as CalaAccountBalance, BalanceRange as CalaBalanceRange},
    primitives::{
        AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, BalanceId as CalaBalanceId,
        EntryId as CalaEntryId, JournalId as CalaJournalId, TransactionId as CalaTxId,
        TxTemplateId as CalaTxTemplateId,
    },
};

pub use money::{Satoshis, UsdCents};

es_entity::entity_id! {
    ChartId,
    ChartNodeId,
    ManualTransactionId,
    LedgerAccountId,
    AccountingCsvId,
    FiscalYearId;

    ChartId => CalaAccountSetId,
    LedgerAccountId => CalaAccountId,
    LedgerAccountId => CalaAccountSetId,
    AccountingCsvId => job::JobId,
}

impl From<cala_ledger::account_set::AccountSetMemberId> for LedgerAccountId {
    fn from(value: cala_ledger::account_set::AccountSetMemberId) -> Self {
        match value {
            cala_ledger::account_set::AccountSetMemberId::Account(id) => id.into(),
            cala_ledger::account_set::AccountSetMemberId::AccountSet(id) => id.into(),
        }
    }
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

pub type LedgerTransactionId = CalaTxId;
pub type TransactionTemplateId = CalaTxTemplateId;

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
impl std::fmt::Display for AccountCodeSection {
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

    pub(super) fn account_set_external_id(&self, chart_id: ChartId) -> String {
        format!("{chart_id}.{self}")
    }

    pub(super) fn manual_account_external_id(&self, chart_id: ChartId) -> String {
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

impl std::fmt::Display for AccountCode {
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

#[derive(Clone, Debug)]
pub enum AccountIdOrCode {
    Id(LedgerAccountId),
    Code(AccountCode),
}

impl std::str::FromStr for AccountIdOrCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<LedgerAccountId>() {
            Ok(AccountIdOrCode::Id(id))
        } else {
            Ok(AccountIdOrCode::Code(s.parse()?))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AccountSpec {
    pub parent: Option<AccountCode>,
    pub code: AccountCode,
    pub name: AccountName,
    pub normal_balance_type: DebitOrCredit,
}

impl AccountSpec {
    pub fn try_new(
        parent: Option<AccountCode>,
        sections: Vec<AccountCodeSection>,
        name: AccountName,
        normal_balance_type: DebitOrCredit,
    ) -> Result<Self, AccountCodeError> {
        let code = AccountCode { sections };
        code.check_valid_parent(parent.clone())?;

        Ok(AccountSpec {
            parent,
            code,
            name,
            normal_balance_type,
        })
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
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

#[derive(Debug, Clone)]
pub struct ResolvedAccountingBaseConfig {
    pub config: AccountingBaseConfig,
    pub assets: CalaAccountSetId,
    pub liabilities: CalaAccountSetId,
    pub equity: CalaAccountSetId,
    pub equity_retained_earnings_gain: CalaAccountSetId,
    pub equity_retained_earnings_loss: CalaAccountSetId,
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub account_set_id: CalaAccountSetId,
    pub code: AccountCode,
    pub name: AccountName,
}

#[derive(Debug, Clone)]
pub struct ClosingAccountCodes {
    pub revenue: AccountCode,
    pub cost_of_revenue: AccountCode,
    pub expenses: AccountCode,
    pub equity_retained_earnings: AccountCode,
    pub equity_retained_losses: AccountCode,
}

impl From<&AccountingBaseConfig> for ClosingAccountCodes {
    fn from(config: &AccountingBaseConfig) -> Self {
        Self {
            revenue: config.revenue_code.clone(),
            cost_of_revenue: config.cost_of_revenue_code.clone(),
            expenses: config.expenses_code.clone(),
            equity_retained_earnings: config.equity_retained_earnings_gain_code.clone(),
            equity_retained_losses: config.equity_retained_earnings_loss_code.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClosingAccountSetIds {
    pub revenue: CalaAccountSetId,
    pub cost_of_revenue: CalaAccountSetId,
    pub expenses: CalaAccountSetId,
    pub equity_retained_earnings: CalaAccountSetId,
    pub equity_retained_losses: CalaAccountSetId,
}

#[derive(Debug, Clone)]
pub struct ClosingTxDetails {
    pub description: String,
    pub tx_id: CalaTxId,
    pub effective_balances_until: chrono::NaiveDate,
    pub effective_balances_from: chrono::NaiveDate,
}

impl ClosingTxDetails {
    pub fn retained_earnings_account_name(&self) -> String {
        format!("NET_INCOME_{}", self.description)
    }
}

pub type ChartAllOrOne = AllOrOne<ChartId>;
pub type JournalAllOrOne = AllOrOne<CalaJournalId>;
pub type LedgerAccountAllOrOne = AllOrOne<LedgerAccountId>;
pub type LedgerTransactionAllOrOne = AllOrOne<CalaTxId>;
pub type TransactionTemplateAllOrOne = AllOrOne<TransactionTemplateId>;
pub type ManualTransactionAllOrOne = AllOrOne<ManualTransactionId>;
pub type ProfitAndLossAllOrOne = AllOrOne<LedgerAccountId>;
pub type ProfitAndLossConfigurationAllOrOne = AllOrOne<LedgerAccountId>;
pub type BalanceSheetAllOrOne = AllOrOne<LedgerAccountId>;
pub type BalanceSheetConfigurationAllOrOne = AllOrOne<LedgerAccountId>;
pub type AccountingCsvAllOrOne = AllOrOne<AccountingCsvId>;
pub type TrialBalanceAllOrOne = AllOrOne<LedgerAccountId>; // what to do if there is only All
// option
pub type FiscalYearAllOrOne = AllOrOne<FiscalYearId>;

pub const PERMISSION_SET_ACCOUNTING_VIEWER: &str = "accounting_viewer";
pub const PERMISSION_SET_ACCOUNTING_WRITER: &str = "accounting_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccountingAction {
    Chart(ChartAction),
    Journal(JournalAction),
    LedgerAccount(LedgerAccountAction),
    LedgerTransaction(LedgerTransactionAction),
    TransactionTemplate(TransactionTemplateAction),
    ManualTransaction(ManualTransactionAction),
    ProfitAndLoss(ProfitAndLossAction),
    ProfitAndLossConfiguration(ProfitAndLossConfigurationAction),
    BalanceSheet(BalanceSheetAction),
    BalanceSheetConfiguration(BalanceSheetConfigurationAction),
    AccountingCsv(AccountingCsvAction),
    TrialBalance(TrialBalanceAction),
    FiscalYear(FiscalYearAction),
}

impl CoreAccountingAction {
    pub fn actions() -> Vec<ActionMapping> {
        use CoreAccountingActionDiscriminants::*;
        use strum::VariantArray;

        CoreAccountingActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Chart => map_action!(accounting, Chart, ChartAction),
                Journal => map_action!(accounting, Journal, JournalAction),
                LedgerAccount => {
                    map_action!(accounting, LedgerAccount, LedgerAccountAction)
                }
                LedgerTransaction => {
                    map_action!(accounting, LedgerTransaction, LedgerTransactionAction)
                }
                TransactionTemplate => {
                    map_action!(accounting, TransactionTemplate, TransactionTemplateAction)
                }
                ManualTransaction => {
                    map_action!(accounting, ManualTransaction, ManualTransactionAction)
                }
                ProfitAndLoss => {
                    map_action!(accounting, ProfitAndLoss, ProfitAndLossAction)
                }
                ProfitAndLossConfiguration => map_action!(
                    accounting,
                    ProfitAndLossConfiguration,
                    ProfitAndLossConfigurationAction
                ),
                BalanceSheet => {
                    map_action!(accounting, BalanceSheet, BalanceSheetAction)
                }
                BalanceSheetConfiguration => map_action!(
                    accounting,
                    BalanceSheetConfiguration,
                    BalanceSheetConfigurationAction
                ),
                AccountingCsv => {
                    map_action!(accounting, AccountingCsv, AccountingCsvAction)
                }
                TrialBalance => {
                    map_action!(accounting, TrialBalance, TrialBalanceAction)
                }
                FiscalYear => {
                    map_action!(accounting, FiscalYear, FiscalYearAction)
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccountingObject {
    Chart(ChartAllOrOne),
    Journal(JournalAllOrOne),
    LedgerAccount(LedgerAccountAllOrOne),
    LedgerTransaction(LedgerTransactionAllOrOne),
    TransactionTemplate(TransactionTemplateAllOrOne),
    ManualTransaction(ManualTransactionAllOrOne),
    ProfitAndLoss(ProfitAndLossAllOrOne),
    ProfitAndLossConfiguration(ProfitAndLossConfigurationAllOrOne),
    BalanceSheet(BalanceSheetAllOrOne),
    BalanceSheetConfiguration(BalanceSheetConfigurationAllOrOne),
    AccountingCsv(AccountingCsvAllOrOne),
    TrialBalance(TrialBalanceAllOrOne),
    FiscalYear(FiscalYearAllOrOne),
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

    pub fn all_transaction_templates() -> Self {
        CoreAccountingObject::TransactionTemplate(AllOrOne::All)
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
        CoreAccountingObject::ProfitAndLossConfiguration(AllOrOne::All)
    }

    pub fn balance_sheet(id: LedgerAccountId) -> Self {
        CoreAccountingObject::BalanceSheet(AllOrOne::ById(id))
    }

    pub fn all_balance_sheet() -> Self {
        CoreAccountingObject::BalanceSheet(AllOrOne::All)
    }

    pub fn all_balance_sheet_configuration() -> Self {
        CoreAccountingObject::BalanceSheetConfiguration(AllOrOne::All)
    }
    pub fn accounting_csv(id: AccountingCsvId) -> Self {
        CoreAccountingObject::AccountingCsv(AllOrOne::ById(id))
    }

    pub fn all_accounting_csvs() -> Self {
        CoreAccountingObject::AccountingCsv(AllOrOne::All)
    }

    pub fn all_trial_balance() -> Self {
        CoreAccountingObject::TrialBalance(AllOrOne::All)
    }
    pub fn fiscal_year(id: FiscalYearId) -> Self {
        CoreAccountingObject::FiscalYear(AllOrOne::ById(id))
    }

    pub fn all_fiscal_years() -> Self {
        CoreAccountingObject::FiscalYear(AllOrOne::All)
    }
}

impl Display for CoreAccountingObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreAccountingObjectDiscriminants::from(self);
        use CoreAccountingObject::*;
        match self {
            Chart(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Journal(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            LedgerAccount(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            LedgerTransaction(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            TransactionTemplate(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            ManualTransaction(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            ProfitAndLoss(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            ProfitAndLossConfiguration(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            BalanceSheet(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            BalanceSheetConfiguration(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            AccountingCsv(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            TrialBalance(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            FiscalYear(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
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
            TransactionTemplate => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse TransactionTemplate")?;
                CoreAccountingObject::TransactionTemplate(obj_ref)
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
            ProfitAndLossConfiguration => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse ProfitAndLossConfiguration")?;
                CoreAccountingObject::ProfitAndLossConfiguration(obj_ref)
            }
            BalanceSheet => {
                let obj_ref = id.parse().map_err(|_| "could not parse BalanceSheet")?;
                CoreAccountingObject::BalanceSheet(obj_ref)
            }
            BalanceSheetConfiguration => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse BalanceSheetConfiguration")?;
                CoreAccountingObject::BalanceSheetConfiguration(obj_ref)
            }
            AccountingCsv => {
                let obj_ref = id.parse().map_err(|_| "could not parse AccountingCsv")?;
                CoreAccountingObject::AccountingCsv(obj_ref)
            }
            TrialBalance => {
                let obj_ref = id.parse().map_err(|_| "could not parse TrialBalance")?;
                CoreAccountingObject::TrialBalance(obj_ref)
            }
            FiscalYear => {
                let obj_ref = id.parse().map_err(|_| "could not parse FiscalYear")?;
                CoreAccountingObject::FiscalYear(obj_ref)
            }
        };
        Ok(res)
    }
}

impl CoreAccountingAction {
    pub const CHART_CREATE: Self = CoreAccountingAction::Chart(ChartAction::Create);
    pub const CHART_LIST: Self = CoreAccountingAction::Chart(ChartAction::List);
    pub const CHART_UPDATE: Self = CoreAccountingAction::Chart(ChartAction::Update);
    pub const CHART_IMPORT_ACCOUNTS: Self =
        CoreAccountingAction::Chart(ChartAction::ImportAccounts);
    pub const CHART_CLOSE_MONTHLY: Self = CoreAccountingAction::Chart(ChartAction::CloseMonthly);

    pub const JOURNAL_READ_ENTRIES: Self =
        CoreAccountingAction::Journal(JournalAction::ReadEntries);

    pub const LEDGER_ACCOUNT_READ: Self =
        CoreAccountingAction::LedgerAccount(LedgerAccountAction::Read);
    pub const LEDGER_ACCOUNT_LIST: Self =
        CoreAccountingAction::LedgerAccount(LedgerAccountAction::List);
    pub const LEDGER_ACCOUNT_READ_HISTORY: Self =
        CoreAccountingAction::LedgerAccount(LedgerAccountAction::ReadHistory);

    pub const LEDGER_TRANSACTION_LIST: Self =
        CoreAccountingAction::LedgerTransaction(LedgerTransactionAction::List);
    pub const LEDGER_TRANSACTION_READ: Self =
        CoreAccountingAction::LedgerTransaction(LedgerTransactionAction::Read);

    pub const TRANSACTION_TEMPLATE_LIST: Self =
        CoreAccountingAction::TransactionTemplate(TransactionTemplateAction::List);

    pub const MANUAL_TRANSACTION_READ: Self =
        CoreAccountingAction::ManualTransaction(ManualTransactionAction::Read);
    pub const MANUAL_TRANSACTION_CREATE: Self =
        CoreAccountingAction::ManualTransaction(ManualTransactionAction::Create);
    pub const MANUAL_TRANSACTION_LIST: Self =
        CoreAccountingAction::ManualTransaction(ManualTransactionAction::List);
    pub const PROFIT_AND_LOSS_READ: Self =
        CoreAccountingAction::ProfitAndLoss(ProfitAndLossAction::Read);
    pub const PROFIT_AND_LOSS_CREATE: Self =
        CoreAccountingAction::ProfitAndLoss(ProfitAndLossAction::Create);
    pub const PROFIT_AND_LOSS_UPDATE: Self =
        CoreAccountingAction::ProfitAndLoss(ProfitAndLossAction::Update);
    pub const PROFIT_AND_LOSS_CONFIGURATION_READ: Self =
        CoreAccountingAction::ProfitAndLossConfiguration(ProfitAndLossConfigurationAction::Read);
    pub const PROFIT_AND_LOSS_CONFIGURATION_UPDATE: Self =
        CoreAccountingAction::ProfitAndLossConfiguration(ProfitAndLossConfigurationAction::Update);

    pub const BALANCE_SHEET_READ: Self =
        CoreAccountingAction::BalanceSheet(BalanceSheetAction::Read);
    pub const BALANCE_SHEET_CREATE: Self =
        CoreAccountingAction::BalanceSheet(BalanceSheetAction::Create);
    pub const BALANCE_SHEET_CONFIGURATION_READ: Self =
        CoreAccountingAction::BalanceSheetConfiguration(BalanceSheetConfigurationAction::Read);
    pub const BALANCE_SHEET_CONFIGURATION_UPDATE: Self =
        CoreAccountingAction::BalanceSheetConfiguration(BalanceSheetConfigurationAction::Update);

    pub const ACCOUNTING_CSV_CREATE: Self =
        CoreAccountingAction::AccountingCsv(AccountingCsvAction::Create);
    pub const ACCOUNTING_CSV_GENERATE: Self =
        CoreAccountingAction::AccountingCsv(AccountingCsvAction::Generate);
    pub const ACCOUNTING_CSV_READ: Self =
        CoreAccountingAction::AccountingCsv(AccountingCsvAction::Read);
    pub const ACCOUNTING_CSV_LIST: Self =
        CoreAccountingAction::AccountingCsv(AccountingCsvAction::List);
    pub const ACCOUNTING_CSV_GENERATE_DOWNLOAD_LINK: Self =
        CoreAccountingAction::AccountingCsv(AccountingCsvAction::Download);

    pub const TRIAL_BALANCE_READ: Self =
        CoreAccountingAction::TrialBalance(TrialBalanceAction::Read);
    pub const TRIAL_BALANCE_CREATE: Self =
        CoreAccountingAction::TrialBalance(TrialBalanceAction::Create);
    pub const TRIAL_BALANCE_UPDATE: Self =
        CoreAccountingAction::TrialBalance(TrialBalanceAction::Update);
    pub const FISCAL_YEAR_READ: Self = CoreAccountingAction::FiscalYear(FiscalYearAction::Read);
    pub const FISCAL_YEAR_LIST: Self = CoreAccountingAction::FiscalYear(FiscalYearAction::List);
    pub const FISCAL_YEAR_CREATE: Self = CoreAccountingAction::FiscalYear(FiscalYearAction::Create);
    pub const FISCAL_YEAR_CLOSE_MONTH: Self =
        CoreAccountingAction::FiscalYear(FiscalYearAction::CloseMonth);
    pub const FISCAL_YEAR_CLOSE: Self = CoreAccountingAction::FiscalYear(FiscalYearAction::Close);
}

impl Display for CoreAccountingAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreAccountingActionDiscriminants::from(self))?;
        use CoreAccountingAction::*;
        match self {
            Chart(action) => action.fmt(f),
            Journal(action) => action.fmt(f),
            LedgerAccount(action) => action.fmt(f),
            LedgerTransaction(action) => action.fmt(f),
            TransactionTemplate(action) => action.fmt(f),
            ManualTransaction(action) => action.fmt(f),
            ProfitAndLoss(action) => action.fmt(f),
            ProfitAndLossConfiguration(action) => action.fmt(f),
            BalanceSheet(action) => action.fmt(f),
            BalanceSheetConfiguration(action) => action.fmt(f),
            AccountingCsv(action) => action.fmt(f),
            TrialBalance(action) => action.fmt(f),
            FiscalYear(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreAccountingAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        let res = match entity.parse()? {
            CoreAccountingActionDiscriminants::Chart => {
                CoreAccountingAction::from(action.parse::<ChartAction>()?)
            }
            CoreAccountingActionDiscriminants::Journal => {
                CoreAccountingAction::from(action.parse::<JournalAction>()?)
            }
            CoreAccountingActionDiscriminants::LedgerAccount => {
                CoreAccountingAction::from(action.parse::<LedgerAccountAction>()?)
            }
            CoreAccountingActionDiscriminants::LedgerTransaction => {
                CoreAccountingAction::from(action.parse::<LedgerTransactionAction>()?)
            }
            CoreAccountingActionDiscriminants::TransactionTemplate => {
                CoreAccountingAction::from(action.parse::<TransactionTemplateAction>()?)
            }
            CoreAccountingActionDiscriminants::ManualTransaction => {
                CoreAccountingAction::from(action.parse::<ManualTransactionAction>()?)
            }
            CoreAccountingActionDiscriminants::ProfitAndLoss => {
                CoreAccountingAction::from(action.parse::<ProfitAndLossAction>()?)
            }
            CoreAccountingActionDiscriminants::ProfitAndLossConfiguration => {
                CoreAccountingAction::from(action.parse::<ProfitAndLossConfigurationAction>()?)
            }
            CoreAccountingActionDiscriminants::BalanceSheet => {
                CoreAccountingAction::from(action.parse::<BalanceSheetAction>()?)
            }
            CoreAccountingActionDiscriminants::BalanceSheetConfiguration => {
                CoreAccountingAction::from(action.parse::<BalanceSheetConfigurationAction>()?)
            }
            CoreAccountingActionDiscriminants::AccountingCsv => {
                CoreAccountingAction::from(action.parse::<AccountingCsvAction>()?)
            }
            CoreAccountingActionDiscriminants::TrialBalance => {
                CoreAccountingAction::from(action.parse::<TrialBalanceAction>()?)
            }
            CoreAccountingActionDiscriminants::FiscalYear => {
                CoreAccountingAction::from(action.parse::<FiscalYearAction>()?)
            }
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ChartAction {
    Create,
    List,
    Update,
    ImportAccounts,
    CloseMonthly,
}

impl ActionPermission for ChartAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List => PERMISSION_SET_ACCOUNTING_VIEWER,

            Self::Create | Self::Update | Self::ImportAccounts | Self::CloseMonthly => {
                PERMISSION_SET_ACCOUNTING_WRITER
            }
        }
    }
}

impl From<ChartAction> for CoreAccountingAction {
    fn from(action: ChartAction) -> Self {
        CoreAccountingAction::Chart(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum LedgerTransactionAction {
    Read,
    List,
    ReadHistory,
}

impl ActionPermission for LedgerTransactionAction {
    fn permission_set(&self) -> &'static str {
        match self {
            // All operations are read-only, use VIEWER permission
            Self::Read | Self::List | Self::ReadHistory => PERMISSION_SET_ACCOUNTING_VIEWER,
        }
    }
}

impl From<LedgerTransactionAction> for CoreAccountingAction {
    fn from(action: LedgerTransactionAction) -> Self {
        CoreAccountingAction::LedgerTransaction(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum LedgerAccountAction {
    Read,
    List,
    ReadHistory,
}

impl ActionPermission for LedgerAccountAction {
    fn permission_set(&self) -> &'static str {
        match self {
            // All operations are read-only, use VIEWER permission
            Self::Read | Self::List | Self::ReadHistory => PERMISSION_SET_ACCOUNTING_VIEWER,
        }
    }
}

impl From<LedgerAccountAction> for CoreAccountingAction {
    fn from(action: LedgerAccountAction) -> Self {
        CoreAccountingAction::LedgerAccount(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum JournalAction {
    ReadEntries,
}

impl ActionPermission for JournalAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::ReadEntries => PERMISSION_SET_ACCOUNTING_VIEWER,
        }
    }
}

impl From<JournalAction> for CoreAccountingAction {
    fn from(action: JournalAction) -> Self {
        CoreAccountingAction::Journal(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum TransactionTemplateAction {
    List,
}

impl ActionPermission for TransactionTemplateAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List => PERMISSION_SET_ACCOUNTING_VIEWER,
        }
    }
}

impl From<TransactionTemplateAction> for CoreAccountingAction {
    fn from(action: TransactionTemplateAction) -> Self {
        CoreAccountingAction::TransactionTemplate(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ManualTransactionAction {
    Read,
    Create,
    List,
}

impl ActionPermission for ManualTransactionAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List | Self::Read => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Create => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<ManualTransactionAction> for CoreAccountingAction {
    fn from(action: ManualTransactionAction) -> Self {
        CoreAccountingAction::ManualTransaction(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ProfitAndLossAction {
    Read,
    Create,
    Update,
}

impl ActionPermission for ProfitAndLossAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Update | Self::Create => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<ProfitAndLossAction> for CoreAccountingAction {
    fn from(action: ProfitAndLossAction) -> Self {
        CoreAccountingAction::ProfitAndLoss(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ProfitAndLossConfigurationAction {
    Read,
    Update,
}

impl ActionPermission for ProfitAndLossConfigurationAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Update => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<ProfitAndLossConfigurationAction> for CoreAccountingAction {
    fn from(action: ProfitAndLossConfigurationAction) -> Self {
        CoreAccountingAction::ProfitAndLossConfiguration(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum BalanceSheetAction {
    Read,
    Create,
}

impl ActionPermission for BalanceSheetAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Create => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<BalanceSheetAction> for CoreAccountingAction {
    fn from(action: BalanceSheetAction) -> Self {
        CoreAccountingAction::BalanceSheet(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum BalanceSheetConfigurationAction {
    Read,
    Update,
}

impl ActionPermission for BalanceSheetConfigurationAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Update => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<BalanceSheetConfigurationAction> for CoreAccountingAction {
    fn from(action: BalanceSheetConfigurationAction) -> Self {
        CoreAccountingAction::BalanceSheetConfiguration(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum AccountingCsvAction {
    Create,
    Generate,
    Read,
    List,
    Download,
}

impl ActionPermission for AccountingCsvAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List | Self::Download => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Create | Self::Generate => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<AccountingCsvAction> for CoreAccountingAction {
    fn from(action: AccountingCsvAction) -> Self {
        CoreAccountingAction::AccountingCsv(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum TrialBalanceAction {
    Create,
    Read,
    Update,
}

impl ActionPermission for TrialBalanceAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Create | Self::Update => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<TrialBalanceAction> for CoreAccountingAction {
    fn from(action: TrialBalanceAction) -> Self {
        CoreAccountingAction::TrialBalance(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum FiscalYearAction {
    Create,
    Read,
    List,
    CloseMonth,
    Close,
}

impl ActionPermission for FiscalYearAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::Create => PERMISSION_SET_ACCOUNTING_WRITER,
            Self::List => PERMISSION_SET_ACCOUNTING_VIEWER,
            Self::CloseMonth => PERMISSION_SET_ACCOUNTING_WRITER,
            Self::Close => PERMISSION_SET_ACCOUNTING_WRITER,
        }
    }
}

impl From<FiscalYearAction> for CoreAccountingAction {
    fn from(action: FiscalYearAction) -> Self {
        CoreAccountingAction::FiscalYear(action)
    }
}

#[derive(Debug, Clone)]
pub struct BalanceRange {
    pub open: Option<CalaAccountBalance>,
    pub close: Option<CalaAccountBalance>,
    pub period_activity: Option<CalaAccountBalance>,
}

impl BalanceRange {
    pub(crate) fn has_non_zero_activity(&self) -> bool {
        if let Some(close) = self.close.as_ref() {
            close.details.settled.dr_balance != Decimal::ZERO
                || close.details.settled.cr_balance != Decimal::ZERO
                || close.details.pending.dr_balance != Decimal::ZERO
                || close.details.pending.cr_balance != Decimal::ZERO
                || close.details.encumbrance.dr_balance != Decimal::ZERO
                || close.details.encumbrance.cr_balance != Decimal::ZERO
        } else {
            false
        }
    }
}

pub use es_entity::clock::ClockHandle;

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn errors_for_new_spec_if_invalid_parent() {
        let parent = "10".parse::<AccountCode>().unwrap();
        let child = "11".parse::<AccountCode>().unwrap();
        let new_spec = AccountSpec::try_new(
            Some(parent),
            child.sections,
            "spec".parse().unwrap(),
            Default::default(),
        );
        assert!(matches!(new_spec, Err(AccountCodeError::InvalidParent)));
    }

    mod is_parent_of {
        use super::*;

        #[test]
        fn not_parent_when_child_sections_empty() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = AccountCode::new(vec![]);
            assert!(!parent.is_parent_of(&child.sections));
        }

        #[test]
        fn not_parent_when_parent_sections_empty() {
            let parent = AccountCode::new(vec![]);
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(&child.sections));
        }

        #[test]
        fn is_parent_when_prefix_matches_in_first_section() {
            let parent = "1".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(parent.is_parent_of(&child.sections));
        }

        #[test]
        fn not_parent_when_prefix_does_not_match_in_first_section() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(&child.sections));
        }

        #[test]
        fn is_parent_when_child_has_more_sections_than_parent() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "10.20".parse::<AccountCode>().unwrap();
            assert!(parent.is_parent_of(&child.sections));

            let parent = "10.20".parse::<AccountCode>().unwrap();
            let child = "10.20.0201".parse::<AccountCode>().unwrap();
            assert!(parent.is_parent_of(&child.sections));
        }

        #[test]
        fn not_parent_when_child_has_more_sections_than_parent() {
            let parent = "10.20".parse::<AccountCode>().unwrap();
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(&child.sections));
        }

        #[test]
        fn not_parent_when_sections_equal() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(&child.sections));
        }

        #[test]
        fn not_parent_when_parent_code_longer_but_prefixed() {
            let parent = "100".parse::<AccountCode>().unwrap();
            let child = "10".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(&child.sections));
        }

        #[test]
        fn not_parent_when_parent_code_longer_but_prefixed_in_second_section() {
            let parent = "1.23".parse::<AccountCode>().unwrap();
            let child = "1.2".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(&child.sections));
        }

        #[test]
        fn not_parent_when_prefix_mismatch_in_second_section() {
            let parent = "1.23".parse::<AccountCode>().unwrap();
            let child = "1.20".parse::<AccountCode>().unwrap();
            assert!(!parent.is_parent_of(&child.sections));
        }
    }

    mod check_valid_parent {
        use super::*;

        #[test]
        fn ok_when_no_parent() {
            let child = "10.20".parse::<AccountCode>().unwrap();
            assert!(child.check_valid_parent(None).is_ok());
        }

        #[test]
        fn ok_when_is_parent() {
            let parent = "1".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(child.check_valid_parent(Some(parent)).is_ok());
        }

        #[test]
        fn err_when_not_parent() {
            let parent = "10".parse::<AccountCode>().unwrap();
            let child = "11".parse::<AccountCode>().unwrap();
            assert!(matches!(
                child.check_valid_parent(Some(parent)),
                Err(AccountCodeError::InvalidParent)
            ));
        }
    }

    mod accounting_base_config {
        use super::*;

        fn default_config() -> AccountingBaseConfig {
            AccountingBaseConfig::try_new(
                "1".parse().unwrap(),
                "2".parse().unwrap(),
                "3".parse().unwrap(),
                "32.01".parse().unwrap(),
                "32.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            )
            .unwrap()
        }

        #[test]
        fn try_new_ok_with_valid_config() {
            let config = default_config();

            assert!(config.assets_code.is_top_level_chart_code());
            assert!(
                config
                    .equity_code
                    .is_parent_of(&config.equity_retained_earnings_gain_code.sections)
            );
        }

        #[test]
        fn try_new_err_when_invalid_config_dup_code() {
            let invalid_config_res = AccountingBaseConfig::try_new(
                "1".parse().unwrap(),
                "1".parse().unwrap(),
                "3".parse().unwrap(),
                "32.01".parse().unwrap(),
                "32.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            );
            assert!(matches!(
                invalid_config_res,
                Err(AccountingBaseConfigError::DuplicateAccountCode(_))
            ))
        }

        #[test]
        fn try_new_err_when_invalid_config_not_top_level() {
            let invalid_config_res = AccountingBaseConfig::try_new(
                "11".parse().unwrap(),
                "2".parse().unwrap(),
                "3".parse().unwrap(),
                "32.01".parse().unwrap(),
                "32.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            );
            assert!(matches!(
                invalid_config_res,
                Err(AccountingBaseConfigError::AccountCodeNotTopLevel(_))
            ))
        }

        #[test]
        fn try_new_err_when_invalid_config_retained_earnings_not_child_of_equity() {
            let invalid_config_res = AccountingBaseConfig::try_new(
                "1".parse().unwrap(),
                "2".parse().unwrap(),
                "3".parse().unwrap(),
                "92.01".parse().unwrap(),
                "92.02".parse().unwrap(),
                "4".parse().unwrap(),
                "5".parse().unwrap(),
                "6".parse().unwrap(),
            );
            assert!(matches!(
                invalid_config_res,
                Err(AccountingBaseConfigError::RetainedEarningsCodeNotChildOfEquity(_))
            ))
        }

        #[test]
        fn is_off_balance_sheet_returns_false_for_configured_codes() {
            let config = default_config();

            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.assets_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.liabilities_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.equity_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.revenue_code));
            assert!(
                !config.is_off_balance_sheet_account_set_or_account(&config.cost_of_revenue_code)
            );
            assert!(!config.is_off_balance_sheet_account_set_or_account(&config.expenses_code));
            assert!(!config.is_off_balance_sheet_account_set_or_account(
                &config.equity_retained_earnings_gain_code
            ));
            assert!(!config.is_off_balance_sheet_account_set_or_account(
                &config.equity_retained_earnings_loss_code
            ));
        }

        #[test]
        fn is_off_balance_sheet_returns_true_for_non_configured_top_level_codes() {
            let config = default_config();
            let code = "9".parse::<AccountCode>().unwrap();
            assert!(config.is_off_balance_sheet_account_set_or_account(&code));
        }

        #[test]
        fn is_off_balance_sheet_returns_true_for_non_configured_child_codes() {
            let config = default_config();
            let code = "91".parse::<AccountCode>().unwrap();
            assert!(config.is_off_balance_sheet_account_set_or_account(&code));
        }

        #[test]
        fn is_assets_returns_true_for_top_level_asset_code() {
            let config = default_config();
            assert!(config.is_assets_account_set_or_account(&config.assets_code));
        }

        #[test]
        fn is_assets_returns_true_for_child_account_set_member() {
            let config = default_config();
            let top_chart_level_account_code = "11".parse::<AccountCode>().unwrap();
            let child_account_code = "11.1".parse::<AccountCode>().unwrap();

            assert!(config.is_assets_account_set_or_account(&top_chart_level_account_code));
            assert!(config.is_assets_account_set_or_account(&child_account_code));
        }

        #[test]
        fn is_assets_returns_false_for_non_asset_code() {
            let config = default_config();
            let off_balance_sheet_code = "9".parse::<AccountCode>().unwrap();
            assert!(!config.is_assets_account_set_or_account(&off_balance_sheet_code));
            assert!(!config.is_assets_account_set_or_account(&config.equity_code));
        }

        #[test]
        fn is_account_in_category_delegates_correctly() {
            let config = default_config();
            let off_balance_sheet_code = "9".parse::<AccountCode>().unwrap();
            let asset_child_code = "11".parse::<AccountCode>().unwrap();

            // OffBalanceSheet
            assert!(
                config.is_account_in_category(
                    &off_balance_sheet_code,
                    AccountCategory::OffBalanceSheet
                )
            );
            assert!(
                !config
                    .is_account_in_category(&config.assets_code, AccountCategory::OffBalanceSheet)
            );

            // Asset
            assert!(config.is_account_in_category(&config.assets_code, AccountCategory::Asset));
            assert!(config.is_account_in_category(&asset_child_code, AccountCategory::Asset));
            assert!(
                !config.is_account_in_category(&config.liabilities_code, AccountCategory::Asset)
            );

            // Liability
            assert!(
                config.is_account_in_category(&config.liabilities_code, AccountCategory::Liability)
            );
            assert!(
                !config.is_account_in_category(&config.assets_code, AccountCategory::Liability)
            );

            // Equity
            assert!(config.is_account_in_category(&config.equity_code, AccountCategory::Equity));
            assert!(config.is_account_in_category(
                &config.equity_retained_earnings_gain_code,
                AccountCategory::Equity
            ));
            assert!(!config.is_account_in_category(&config.assets_code, AccountCategory::Equity));

            // Revenue
            assert!(config.is_account_in_category(&config.revenue_code, AccountCategory::Revenue));
            assert!(!config.is_account_in_category(&config.assets_code, AccountCategory::Revenue));
        }
    }
}
