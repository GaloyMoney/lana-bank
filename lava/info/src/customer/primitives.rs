use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CustomerInfoModuleAction {
    CustomerInfo(CustomerInfoAction),
}

impl CustomerInfoModuleAction {
    pub const CUSTOMER_INFO_READ: Self =
        CustomerInfoModuleAction::CustomerInfo(CustomerInfoAction::Read);
}

impl Display for CustomerInfoModuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CustomerInfoModuleActionDiscriminants::from(self))?;
        use CustomerInfoModuleAction::*;
        match self {
            CustomerInfo(action) => action.fmt(f),
        }
    }
}

impl FromStr for CustomerInfoModuleAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CustomerInfoModuleActionDiscriminants::*;
        let res = match entity.parse()? {
            CustomerInfo => action.parse::<CustomerInfoAction>()?,
        };
        Ok(res.into())
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CustomerInfoAction {
    Read,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CustomerInfoModuleObject {
    CustomerInfo,
}

impl From<CustomerInfoAction> for CustomerInfoModuleAction {
    fn from(action: CustomerInfoAction) -> Self {
        CustomerInfoModuleAction::CustomerInfo(action)
    }
}
