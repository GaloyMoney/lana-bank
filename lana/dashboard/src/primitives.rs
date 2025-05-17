use std::{fmt::Display, str::FromStr};

use authz::permission_set::{ActionDescription, NoPath};

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DashboardModuleAction {
    Dashboard(DashboardAction),
}

impl DashboardModuleAction {
    pub const DASHBOARD_READ: Self = DashboardModuleAction::Dashboard(DashboardAction::Read);

    pub fn entities() -> Vec<(
        DashboardModuleActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use DashboardModuleActionDiscriminants::*;

        let mut result = vec![];

        for entity in <DashboardModuleActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                Dashboard => DashboardAction::describe(),
            };

            result.push((*entity, actions));
        }
        result
    }
}

impl Display for DashboardModuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", DashboardModuleActionDiscriminants::from(self))?;
        use DashboardModuleAction::*;
        match self {
            Dashboard(action) => action.fmt(f),
        }
    }
}

impl FromStr for DashboardModuleAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use DashboardModuleActionDiscriminants::*;
        let res = match entity.parse()? {
            Dashboard => action.parse::<DashboardAction>()?,
        };
        Ok(res.into())
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DashboardAction {
    Read,
}

impl DashboardAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let set = match variant {
                Self::Read => vec!["set1"],
            };
            res.push(ActionDescription::new(variant, set));
        }

        res
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum DashboardModuleObject {
    Dashboard,
}

impl From<DashboardAction> for DashboardModuleAction {
    fn from(action: DashboardAction) -> Self {
        DashboardModuleAction::Dashboard(action)
    }
}
