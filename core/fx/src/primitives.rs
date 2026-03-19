use std::{fmt::Display, str::FromStr};

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};

pub use cala_ledger::Currency;
pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, JournalId as CalaJournalId,
    TransactionId as CalaTransactionId,
};
pub use chart_primitives::ChartId;

use crate::error::CoreFxError;

es_entity::entity_id! {
    ChartOfAccountsIntegrationConfigId,
    FxPositionId;
}

/// Exchange rate: 1 unit of base currency = rate units of quote currency.
/// Example: USD/EUR rate of 0.91 means 1 USD = 0.91 EUR.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExchangeRate {
    rate: Decimal,
    target_precision: u32,
}

impl ExchangeRate {
    pub fn try_new(rate: Decimal, target_precision: u32) -> Result<Self, CoreFxError> {
        if rate <= Decimal::ZERO {
            return Err(CoreFxError::InvalidExchangeRate);
        }
        Ok(Self {
            rate,
            target_precision,
        })
    }

    pub fn rate(&self) -> Decimal {
        self.rate
    }

    pub fn inverse(&self, source_precision: u32) -> Self {
        Self {
            rate: Decimal::ONE / self.rate,
            target_precision: source_precision,
        }
    }

    /// Compute target_amount = source_amount * rate, rounded down (conservative for the bank).
    /// Returns (target_amount, rounding_difference in target currency).
    pub fn convert(&self, source_amount: Decimal) -> (Decimal, Decimal) {
        let exact = source_amount * self.rate;
        let rounded = exact.round_dp_with_strategy(self.target_precision, RoundingStrategy::ToZero);
        let rounding_diff = exact - rounded;
        (rounded, rounding_diff)
    }
}

/// Result of an FX conversion including any realized gain/loss and rounding.
#[derive(Debug, Clone)]
pub struct FxConversionResult {
    pub target_amount: Decimal,
    pub rounding_difference: Decimal,
    pub realized_gain_loss: Decimal,
}

pub const FX_TRANSACTION_ENTITY_TYPE: chart_primitives::EntityType =
    chart_primitives::EntityType::new("FxConversion");

pub type ChartOfAccountsIntegrationConfigAllOrOne = AllOrOne<ChartOfAccountsIntegrationConfigId>;

permission_sets_macro::permission_sets! {
    FxViewer("Can view FX chart of accounts integration configuration"),
    FxWriter("Can update FX chart of accounts integration configuration"),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreFxObject {
    ChartOfAccountsIntegrationConfig(ChartOfAccountsIntegrationConfigAllOrOne),
}

impl CoreFxObject {
    pub fn chart_of_accounts_integration() -> Self {
        CoreFxObject::ChartOfAccountsIntegrationConfig(AllOrOne::All)
    }
}

impl Display for CoreFxObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreFxObjectDiscriminants::from(self);
        use CoreFxObject::*;
        match self {
            ChartOfAccountsIntegrationConfig(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreFxObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreFxObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            ChartOfAccountsIntegrationConfig => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreFxObject")?;
                CoreFxObject::ChartOfAccountsIntegrationConfig(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreFxAction {
    ChartOfAccountsIntegrationConfig(ChartOfAccountsIntegrationConfigAction),
}

impl CoreFxAction {
    pub const CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE: Self =
        CoreFxAction::ChartOfAccountsIntegrationConfig(
            ChartOfAccountsIntegrationConfigAction::Update,
        );
    pub const CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ: Self =
        CoreFxAction::ChartOfAccountsIntegrationConfig(
            ChartOfAccountsIntegrationConfigAction::Read,
        );

    pub fn actions() -> Vec<ActionMapping> {
        use CoreFxActionDiscriminants::*;
        use strum::VariantArray;

        CoreFxActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                ChartOfAccountsIntegrationConfig => map_action!(
                    fx,
                    ChartOfAccountsIntegrationConfig,
                    ChartOfAccountsIntegrationConfigAction
                ),
            })
            .collect()
    }
}

impl Display for CoreFxAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreFxActionDiscriminants::from(self))?;
        use CoreFxAction::*;
        match self {
            ChartOfAccountsIntegrationConfig(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreFxAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreFxActionDiscriminants::*;
        let res = match entity.parse()? {
            ChartOfAccountsIntegrationConfig => {
                CoreFxAction::from(action.parse::<ChartOfAccountsIntegrationConfigAction>()?)
            }
        };

        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ChartOfAccountsIntegrationConfigAction {
    Read,
    Update,
}

impl ActionPermission for ChartOfAccountsIntegrationConfigAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_FX_VIEWER,
            Self::Update => PERMISSION_SET_FX_WRITER,
        }
    }
}

impl From<ChartOfAccountsIntegrationConfigAction> for CoreFxAction {
    fn from(action: ChartOfAccountsIntegrationConfigAction) -> Self {
        CoreFxAction::ChartOfAccountsIntegrationConfig(action)
    }
}
