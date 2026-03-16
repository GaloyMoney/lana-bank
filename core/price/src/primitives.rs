use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};
use std::fmt;

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};
use money::{Satoshis, UsdCents};

es_entity::entity_id! {
    PriceProviderId
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PriceOfOneBTC(UsdCents);

impl PriceOfOneBTC {
    pub const ZERO: Self = Self::new(UsdCents::ZERO);

    pub const fn new(price: UsdCents) -> Self {
        Self(price)
    }

    pub fn cents_to_sats_round_up(self, cents: UsdCents) -> Satoshis {
        let btc = (cents.to_usd() / self.0.to_usd())
            .round_dp_with_strategy(8, RoundingStrategy::AwayFromZero);
        Satoshis::try_from_btc(btc).expect("Decimal should have no fractional component here")
    }

    pub fn sats_to_cents_round_down(self, sats: Satoshis) -> UsdCents {
        let usd =
            (sats.to_btc() * self.0.to_usd()).round_dp_with_strategy(2, RoundingStrategy::ToZero);
        UsdCents::try_from_usd(usd).expect("Decimal should have no fractional component here")
    }

    pub fn into_inner(self) -> UsdCents {
        self.0
    }
}

impl fmt::Display for PriceOfOneBTC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0.to_usd())
    }
}

permission_sets_macro::permission_sets! {
    PriceViewer("Can view price provider configurations"),
    PriceWriter("Can create and manage price provider configurations"),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CorePriceAction {
    Provider(ProviderAction),
}

impl CorePriceAction {
    pub const PROVIDER_CREATE: Self = CorePriceAction::Provider(ProviderAction::Create);
    pub const PROVIDER_LIST: Self = CorePriceAction::Provider(ProviderAction::List);
    pub const PROVIDER_UPDATE: Self = CorePriceAction::Provider(ProviderAction::Update);

    pub fn actions() -> Vec<ActionMapping> {
        use CorePriceActionDiscriminants::*;
        use strum::VariantArray;

        CorePriceActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Provider => map_action!(price, Provider, ProviderAction),
            })
            .collect()
    }
}

impl core::fmt::Display for CorePriceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CorePriceActionDiscriminants::from(self))?;
        match self {
            Self::Provider(action) => action.fmt(f),
        }
    }
}

impl core::str::FromStr for CorePriceAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CorePriceActionDiscriminants::*;
        let res = match entity.parse()? {
            Provider => CorePriceAction::from(action.parse::<ProviderAction>()?),
        };

        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ProviderAction {
    Create,
    List,
    Update,
}

impl ActionPermission for ProviderAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Create | Self::Update => PERMISSION_SET_PRICE_WRITER,
            Self::List => PERMISSION_SET_PRICE_VIEWER,
        }
    }
}

impl From<ProviderAction> for CorePriceAction {
    fn from(action: ProviderAction) -> Self {
        Self::Provider(action)
    }
}

pub type PriceProviderAllOrOne = AllOrOne<PriceProviderId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CorePriceObject {
    Provider(PriceProviderAllOrOne),
}

impl CorePriceObject {
    pub const fn all_providers() -> Self {
        CorePriceObject::Provider(AllOrOne::All)
    }

    pub const fn provider(id: PriceProviderId) -> Self {
        CorePriceObject::Provider(AllOrOne::ById(id))
    }
}

impl core::fmt::Display for CorePriceObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CorePriceObjectDiscriminants::from(self);
        match self {
            Self::Provider(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl core::str::FromStr for CorePriceObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CorePriceObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Provider => {
                let obj_ref = id.parse().map_err(|_| "could not parse CorePriceObject")?;
                Self::Provider(obj_ref)
            }
        };
        Ok(res)
    }
}
