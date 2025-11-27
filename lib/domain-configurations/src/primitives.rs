use authz::{ActionPermission, action_description::*};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref, str::FromStr};

pub const PERMISSION_SET_DOMAIN_CONFIGURATION_VIEWER: &str = "domain-configuration-viewer";
pub const PERMISSION_SET_DOMAIN_CONFIGURATION_WRITER: &str = "domain-configuration-writer";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainConfigurationKey(String);

impl DomainConfigurationKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl From<&str> for DomainConfigurationKey {
    fn from(value: &str) -> Self {
        DomainConfigurationKey::new(value)
    }
}

impl From<String> for DomainConfigurationKey {
    fn from(value: String) -> Self {
        DomainConfigurationKey::new(value)
    }
}

impl Display for DomainConfigurationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for DomainConfigurationKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for DomainConfigurationKey {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DomainConfigurationKey::new(s.to_string()))
    }
}

/// Marker trait associating a string key with a typed value for configs.
pub trait ConfigKey<T> {
    fn key() -> DomainConfigurationKey;
    fn object() -> DomainConfigurationObject;
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DomainConfigurationEntityAction {
    Read,
    Write,
}

impl ActionPermission for DomainConfigurationEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_DOMAIN_CONFIGURATION_VIEWER,
            Self::Write => PERMISSION_SET_DOMAIN_CONFIGURATION_WRITER,
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, strum::EnumDiscriminants,
    strum::VariantArray, strum::Display, strum::EnumString,
)]
#[strum(serialize_all = "kebab-case")]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DomainConfigurationObject {
    DepositChart,
    CreditChart,
    BalanceSheetChart,
    ProfitAndLossChart,
}

impl Display for DomainConfigurationObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/*", DomainConfigurationObjectDiscriminants::from(self))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DomainConfigurationAction {
    DepositChart(DomainConfigurationEntityAction),
    CreditChart(DomainConfigurationEntityAction),
    BalanceSheetChart(DomainConfigurationEntityAction),
    ProfitAndLossChart(DomainConfigurationEntityAction),
}

impl DomainConfigurationAction {
    pub const fn new(
        object: DomainConfigurationObject,
        action: DomainConfigurationEntityAction,
    ) -> Self {
        use DomainConfigurationAction::*;
        use DomainConfigurationObject::*;
        match object {
            DepositChart => DepositChart(action),
            CreditChart => CreditChart(action),
            BalanceSheetChart => BalanceSheetChart(action),
            ProfitAndLossChart => ProfitAndLossChart(action),
        }
    }

    pub fn actions() -> Vec<ActionMapping> {
        use DomainConfigurationObject::*;
        use DomainConfigurationEntityAction::*;
        let entities = [
            DepositChart,
            CreditChart,
            BalanceSheetChart,
            ProfitAndLossChart,
        ];
        let actions = [Read, Write];
        let module = "domain-config";

        let mut mappings = Vec::new();
        for entity in entities {
            let entity_name = DomainConfigurationObjectDiscriminants::from(&entity).to_string();
            for action in actions {
                mappings.push(ActionMapping::new(
                    module,
                    &entity_name,
                    action,
                    action.permission_set(),
                ));
            }
        }
        mappings
    }
}

impl Display for DomainConfigurationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:",
            DomainConfigurationActionDiscriminants::from(self)
        )?;
        use DomainConfigurationAction::*;
        match self {
            DepositChart(action)
            | CreditChart(action)
            | BalanceSheetChart(action)
            | ProfitAndLossChart(action) => action.fmt(f),
        }
    }
}

impl FromStr for DomainConfigurationAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use DomainConfigurationActionDiscriminants::*;
        let res = match entity.parse()? {
            DepositChart => DomainConfigurationAction::new(
                DomainConfigurationObject::DepositChart,
                action.parse::<DomainConfigurationEntityAction>()?,
            ),
            CreditChart => DomainConfigurationAction::new(
                DomainConfigurationObject::CreditChart,
                action.parse::<DomainConfigurationEntityAction>()?,
            ),
            BalanceSheetChart => DomainConfigurationAction::new(
                DomainConfigurationObject::BalanceSheetChart,
                action.parse::<DomainConfigurationEntityAction>()?,
            ),
            ProfitAndLossChart => DomainConfigurationAction::new(
                DomainConfigurationObject::ProfitAndLossChart,
                action.parse::<DomainConfigurationEntityAction>()?,
            ),
        };

        Ok(res)
    }
}

impl From<DomainConfigurationObject> for DomainConfigurationAction {
    fn from(object: DomainConfigurationObject) -> Self {
        DomainConfigurationAction::new(object, DomainConfigurationEntityAction::Read)
    }
}
