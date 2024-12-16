use std::{fmt::Display, str::FromStr};

use authz::AllOrOne;
use serde::{Deserialize, Serialize};
use sqlx::types::uuid::uuid;

es_entity::entity_id! {
    ChartOfAccountId,
}

impl Default for ChartOfAccountId {
    fn default() -> Self {
        Self(uuid!("00000000-0000-0000-0000-000000000001"))
    }
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
    pub fn chart_of_account() -> Self {
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
    pub const CHART_OF_ACCOUNT_FIND_OR_CREATE: Self =
        CoreChartOfAccountAction::ChartOfAccount(ChartOfAccountAction::FindOrCreate);
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
    FindOrCreate,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash, Deserialize)]
pub struct AccountIdx(u64);
impl Display for AccountIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl From<u32> for AccountIdx {
    fn from(num: u32) -> Self {
        Self(num.into())
    }
}

impl AccountIdx {
    pub const FIRST: Self = Self(1);
    pub const MAX_TWO_DIGIT: Self = Self(99);
    pub const MAX_THREE_DIGIT: Self = Self(999);

    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
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

impl Display for ChartOfAccountCategoryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assets => write!(f, "Assets"),
            Self::Liabilities => write!(f, "Liabilities"),
            Self::Equity => write!(f, "Equity"),
            Self::Revenues => write!(f, "Revenues"),
            Self::Expenses => write!(f, "Expenses"),
        }
    }
}
