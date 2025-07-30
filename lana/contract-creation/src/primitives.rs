use std::{fmt::Display, str::FromStr};

use authz::{AllOrOne, action_description::*};
use lana_ids::ContractCreationId;

pub const PERMISSION_SET_CONTRACT_CREATION: &str = "contract_creation";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum ContractModuleAction {
    Contract(ContractAction),
}

impl ContractModuleAction {
    pub const CONTRACT_CREATE: Self = ContractModuleAction::Contract(ContractAction::Create);

    pub fn entities() -> Vec<(
        ContractModuleActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use ContractModuleActionDiscriminants::*;

        let mut result = vec![];

        for entity in <ContractModuleActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                Contract => ContractAction::describe(),
            };

            result.push((*entity, actions));
        }
        result
    }
}

impl Display for ContractModuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", ContractModuleActionDiscriminants::from(self))?;
        use ContractModuleAction::*;
        match self {
            Contract(action) => action.fmt(f),
        }
    }
}

impl FromStr for ContractModuleAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use ContractModuleActionDiscriminants::*;
        let res = match entity.parse()? {
            Contract => action.parse::<ContractAction>()?,
        };
        Ok(res.into())
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ContractAction {
    Create,
}

impl ContractAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CONTRACT_CREATION])
                }
            };
            res.push(action_description);
        }

        res
    }
}

pub type ContractAllOrOne = AllOrOne<ContractCreationId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum ContractModuleObject {
    Contract(ContractAllOrOne),
}

impl ContractModuleObject {
    pub const fn all_contracts() -> Self {
        Self::Contract(AllOrOne::All)
    }
}

impl From<ContractAction> for ContractModuleAction {
    fn from(action: ContractAction) -> Self {
        ContractModuleAction::Contract(action)
    }
}

impl std::fmt::Display for ContractModuleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = ContractModuleObjectDiscriminants::from(self);
        match self {
            Self::Contract(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for ContractModuleObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use ContractModuleObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Contract => {
                let obj_ref = id.parse().map_err(|_| "could not parse ContractObject")?;
                ContractModuleObject::Contract(obj_ref)
            }
        };
        Ok(res)
    }
}
