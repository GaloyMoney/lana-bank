use std::{fmt::Display, str::FromStr};

use authz::AllOrOne;
use serde::{Deserialize, Serialize};

pub use cala_ledger::{primitives::AccountId as LedgerAccountId, DebitOrCredit};

pub use crate::code::{ChartOfAccountCategoryCode as CategoryPath, ChartOfAccountCode};

es_entity::entity_id! {
    ChartId,
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
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreChartOfAccountObject")?;
                CoreChartOfAccountsObject::Chart(obj_ref)
            }
        };
        Ok(res)
    }
}

impl CoreChartOfAccountsAction {
    pub const CHART_CREATE: Self = CoreChartOfAccountsAction::ChartAction(ChartAction::Create);
    pub const CHART_LIST: Self = CoreChartOfAccountsAction::ChartAction(ChartAction::List);
    pub const CHART_CREATE_CONTROL_ACCOUNT: Self =
        CoreChartOfAccountsAction::ChartAction(ChartAction::CreateControlAccount);
    pub const CHART_FIND_CONTROL_ACCOUNT: Self =
        CoreChartOfAccountsAction::ChartAction(ChartAction::FindControlAccount);
    pub const CHART_CREATE_CONTROL_SUB_ACCOUNT: Self =
        CoreChartOfAccountsAction::ChartAction(ChartAction::CreateControlSubAccount);
    pub const CHART_FIND_CONTROL_SUB_ACCOUNT: Self =
        CoreChartOfAccountsAction::ChartAction(ChartAction::FindControlSubAccount);
    pub const CHART_FIND_TRANSACTION_ACCOUNT: Self =
        CoreChartOfAccountsAction::ChartAction(ChartAction::FindTransactionAccount);
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
    FindTransactionAccount,
}

impl From<ChartAction> for CoreChartOfAccountsAction {
    fn from(action: ChartAction) -> Self {
        CoreChartOfAccountsAction::ChartAction(action)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOfAccountAccountDetails {
    pub account_id: LedgerAccountId,
    pub path: ChartOfAccountCode,
    pub code: String,
    pub name: String,
    pub description: String,
}
