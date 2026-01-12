use authz::{ActionPermission, action_description::*, map_action};
use std::{fmt::Display, str::FromStr};

pub const PERMISSION_SET_EXPOSED_CONFIGS_VIEWER: &str = "exposed_configs_viewer";
pub const PERMISSION_SET_EXPOSED_CONFIGS_WRITER: &str = "exposed_configs_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DomainConfigAction {
    ExposedConfig(DomainConfigEntityAction),
}

impl DomainConfigAction {
    pub const EXPOSED_CONFIG_READ: Self =
        DomainConfigAction::ExposedConfig(DomainConfigEntityAction::Read);
    pub const EXPOSED_CONFIG_WRITE: Self =
        DomainConfigAction::ExposedConfig(DomainConfigEntityAction::Write);

    pub fn actions() -> Vec<ActionMapping> {
        use DomainConfigActionDiscriminants::*;
        map_action!(domain_config, ExposedConfig, DomainConfigEntityAction)
    }
}

impl Display for DomainConfigAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", DomainConfigActionDiscriminants::from(self))?;
        use DomainConfigAction::*;
        match self {
            ExposedConfig(action) => action.fmt(f),
        }
    }
}

impl FromStr for DomainConfigAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use DomainConfigActionDiscriminants::*;
        let res = match entity.parse()? {
            ExposedConfig => DomainConfigAction::from(action.parse::<DomainConfigEntityAction>()?),
        };
        Ok(res)
    }
}

#[derive(Clone, PartialEq, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DomainConfigEntityAction {
    Read,
    Write,
}

impl ActionPermission for DomainConfigEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_EXPOSED_CONFIGS_VIEWER,
            Self::Write => PERMISSION_SET_EXPOSED_CONFIGS_WRITER,
        }
    }
}

impl From<DomainConfigEntityAction> for DomainConfigAction {
    fn from(action: DomainConfigEntityAction) -> Self {
        DomainConfigAction::ExposedConfig(action)
    }
}
