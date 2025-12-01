use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

use crate::primitives::CollateralId;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollateralAction {
    Collateral(CollateralAction),
}

impl CoreCreditCollateralAction {
    pub const COLLATERAL_UPDATE: Self =
        CoreCreditCollateralAction::Collateral(CollateralAction::Update);
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CollateralAction{
    Update,
}

pub type CollateralAllOrOne = AllOrOne<CollateralId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollateralObject {
    Collateral(CollateralAllOrOne),
}

impl CoreCreditCollateralObject {
    pub fn all_proposals() -> Self {
        CoreCreditCollateralObject::Collateral(AllOrOne::All)
    }
}
