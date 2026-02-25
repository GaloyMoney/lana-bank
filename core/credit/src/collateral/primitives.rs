use std::str::FromStr;

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

use crate::{CollateralId, LiquidationId};

es_entity::entity_id! {
    SecuredLoanId,
}

pub type CollateralAllOrOne = AllOrOne<CollateralId>;
pub type LiquidationAllOrOne = AllOrOne<LiquidationId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollateralObject {
    Collateral(CollateralAllOrOne),
    Liquidation(LiquidationAllOrOne),
}

impl CoreCreditCollateralObject {
    pub fn collateral(id: CollateralId) -> Self {
        CoreCreditCollateralObject::Collateral(AllOrOne::ById(id))
    }

    pub fn all_collaterals() -> Self {
        CoreCreditCollateralObject::Collateral(AllOrOne::All)
    }

    pub fn liquidation(id: LiquidationId) -> Self {
        CoreCreditCollateralObject::Liquidation(AllOrOne::ById(id))
    }

    pub fn all_liquidations() -> Self {
        CoreCreditCollateralObject::Liquidation(AllOrOne::All)
    }
}

impl std::fmt::Display for CoreCreditCollateralObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCreditCollateralObjectDiscriminants::from(self);
        use CoreCreditCollateralObject::*;
        match self {
            Collateral(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Liquidation(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreCreditCollateralObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCreditCollateralObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Collateral => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCreditCollateralObject")?;
                CoreCreditCollateralObject::Collateral(obj_ref)
            }
            Liquidation => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCreditCollateralObject")?;
                CoreCreditCollateralObject::Liquidation(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollateralAction {
    Collateral(CollateralAction),
    Liquidation(LiquidationAction),
}

impl CoreCreditCollateralAction {
    pub const COLLATERAL_RECORD_MANUAL_UPDATE: Self =
        CoreCreditCollateralAction::Collateral(CollateralAction::RecordManualUpdate);
    pub const COLLATERAL_RECORD_CUSTODIAN_SYNC: Self =
        CoreCreditCollateralAction::Collateral(CollateralAction::RecordCustodianSync);
    pub const COLLATERAL_RECORD_LIQUIDATION_UPDATE: Self =
        CoreCreditCollateralAction::Collateral(CollateralAction::RecordLiquidationUpdate);
    pub const COLLATERAL_RECORD_PAYMENT_RECEIVED_FROM_LIQUIDATION: Self =
        CoreCreditCollateralAction::Collateral(
            CollateralAction::RecordPaymentReceivedFromLiquidation,
        );
    pub const LIQUIDATION_READ: Self =
        CoreCreditCollateralAction::Liquidation(LiquidationAction::Read);
    pub const LIQUIDATION_LIST: Self =
        CoreCreditCollateralAction::Liquidation(LiquidationAction::List);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCreditCollateralActionDiscriminants::*;
        use strum::VariantArray;

        CoreCreditCollateralActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Collateral => map_action!(credit_collateral, Collateral, CollateralAction),
                Liquidation => map_action!(credit_collateral, Liquidation, LiquidationAction),
            })
            .collect()
    }
}

impl std::fmt::Display for CoreCreditCollateralAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:",
            CoreCreditCollateralActionDiscriminants::from(self)
        )?;
        use CoreCreditCollateralAction::*;
        match self {
            Collateral(action) => action.fmt(f),
            Liquidation(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreCreditCollateralAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreCreditCollateralActionDiscriminants::*;
        let res = match entity.parse()? {
            Collateral => CoreCreditCollateralAction::Collateral(action.parse()?),
            Liquidation => CoreCreditCollateralAction::Liquidation(action.parse()?),
        };
        Ok(res)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CollateralAction {
    RecordManualUpdate,
    RecordCustodianSync,
    RecordLiquidationUpdate,
    RecordPaymentReceivedFromLiquidation,
}

impl ActionPermission for CollateralAction {
    fn permission_set(&self) -> &'static str {
        "CreditWriter"
    }
}

impl From<CollateralAction> for CoreCreditCollateralAction {
    fn from(action: CollateralAction) -> Self {
        Self::Collateral(action)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum LiquidationAction {
    Read,
    List,
}

impl ActionPermission for LiquidationAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => "CreditViewer",
            Self::List => "CreditViewer",
        }
    }
}

impl From<LiquidationAction> for CoreCreditCollateralAction {
    fn from(action: LiquidationAction) -> Self {
        Self::Liquidation(action)
    }
}
