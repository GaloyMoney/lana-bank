use std::{fmt::Display, str::FromStr};

use authz::AllOrOne;
use serde::{Deserialize, Serialize};

pub use cala_ledger::primitives::AccountId as LedgerAccountId;

use crate::code::ChartOfAccountCode;

es_entity::entity_id! {
    ChartOfAccountId,
}

pub type ChartOfAccountAllOrOne = AllOrOne<ChartOfAccountId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreChartOfAccountAction {
    ChartOfAccount(ChartOfAccountAction),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreChartOfAccountObject {
    ChartOfAccount(ChartOfAccountAllOrOne),
}

impl CoreChartOfAccountObject {
    pub fn chart(id: ChartOfAccountId) -> Self {
        CoreChartOfAccountObject::ChartOfAccount(AllOrOne::ById(id))
    }

    pub fn all_charts() -> Self {
        CoreChartOfAccountObject::ChartOfAccount(AllOrOne::All)
    }
}

impl Display for CoreChartOfAccountObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreChartOfAccountObjectDiscriminants::from(self);
        use CoreChartOfAccountObject::*;
        match self {
            ChartOfAccount(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CoreChartOfAccountObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreChartOfAccountObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            ChartOfAccount => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreChartOfAccountObject")?;
                CoreChartOfAccountObject::ChartOfAccount(obj_ref)
            }
        };
        Ok(res)
    }
}

impl CoreChartOfAccountAction {
    pub const CHART_OF_ACCOUNT_CREATE: Self =
        CoreChartOfAccountAction::ChartOfAccount(ChartOfAccountAction::Create);
    pub const CHART_OF_ACCOUNT_LIST: Self =
        CoreChartOfAccountAction::ChartOfAccount(ChartOfAccountAction::List);
    pub const CHART_OF_ACCOUNT_CREATE_CONTROL_ACCOUNT: Self =
        CoreChartOfAccountAction::ChartOfAccount(ChartOfAccountAction::CreateControlAccount);
    pub const CHART_OF_ACCOUNT_CREATE_CONTROL_SUB_ACCOUNT: Self =
        CoreChartOfAccountAction::ChartOfAccount(ChartOfAccountAction::CreateControlSubAccount);
    pub const CHART_OF_ACCOUNT_CREATE_TRANSACTION_ACCOUNT: Self =
        CoreChartOfAccountAction::ChartOfAccount(ChartOfAccountAction::CreateTransactionAccount);
    pub const CHART_OF_ACCOUNT_FIND_TRANSACTION_ACCOUNT: Self =
        CoreChartOfAccountAction::ChartOfAccount(ChartOfAccountAction::FindTransactionAccount);
}

impl Display for CoreChartOfAccountAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreChartOfAccountActionDiscriminants::from(self))?;
        use CoreChartOfAccountAction::*;
        match self {
            ChartOfAccount(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreChartOfAccountAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreChartOfAccountActionDiscriminants::*;
        let res = match entity.parse()? {
            ChartOfAccount => {
                CoreChartOfAccountAction::from(action.parse::<ChartOfAccountAction>()?)
            }
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ChartOfAccountAction {
    Create,
    List,
    CreateControlAccount,
    CreateControlSubAccount,
    CreateTransactionAccount,
    FindTransactionAccount,
}

impl From<ChartOfAccountAction> for CoreChartOfAccountAction {
    fn from(action: ChartOfAccountAction) -> Self {
        CoreChartOfAccountAction::ChartOfAccount(action)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOfAccountAccountDetails {
    pub account_id: LedgerAccountId,
    pub code: ChartOfAccountCode,
    pub name: String,
    pub description: String,
}
