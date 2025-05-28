use std::{fmt::Display, str::FromStr};

use authz::{action_description::*, AllOrOne};

pub const PERMISSION_SET_DASHBOARD_VIEWER: &str = "dashboard_viewer";

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
            let action_description = match variant {
                Self::Read => ActionDescription::new(variant, &[PERMISSION_SET_DASHBOARD_VIEWER]),
            };
            res.push(action_description);
        }

        res
    }
}

es_entity::entity_id!(DashboardId);

pub type DashboardAllOrOne = AllOrOne<DashboardId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DashboardModuleObject {
    Dashboard(DashboardAllOrOne),
}

impl DashboardModuleObject {
    pub const fn all_dashboards() -> Self {
        Self::Dashboard(AllOrOne::All)
    }
}

impl From<DashboardAction> for DashboardModuleAction {
    fn from(action: DashboardAction) -> Self {
        DashboardModuleAction::Dashboard(action)
    }
}

impl std::fmt::Display for DashboardModuleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = DashboardModuleObjectDiscriminants::from(self);
        match self {
            Self::Dashboard(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for DashboardModuleObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use DashboardModuleObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Dashboard => {
                let obj_ref = id.parse().map_err(|_| "could not parse DashboardObject")?;
                DashboardModuleObject::Dashboard(obj_ref)
            }
        };
        Ok(res)
    }
}
