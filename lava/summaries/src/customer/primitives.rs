use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CustomerSummaryModuleAction {
    CustomerSummary(CustomerSummaryAction),
}

impl CustomerSummaryModuleAction {
    pub const CUSTOMER_SUMMARY_READ: Self =
        CustomerSummaryModuleAction::CustomerSummary(CustomerSummaryAction::Read);
}

impl Display for CustomerSummaryModuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:",
            CustomerSummaryModuleActionDiscriminants::from(self)
        )?;
        use CustomerSummaryModuleAction::*;
        match self {
            CustomerSummary(action) => action.fmt(f),
        }
    }
}

impl FromStr for CustomerSummaryModuleAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CustomerSummaryModuleActionDiscriminants::*;
        let res = match entity.parse()? {
            CustomerSummary => action.parse::<CustomerSummaryAction>()?,
        };
        Ok(res.into())
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CustomerSummaryAction {
    Read,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CustomerSummaryModuleObject {
    CustomerSummary,
}

impl From<CustomerSummaryAction> for CustomerSummaryModuleAction {
    fn from(action: CustomerSummaryAction) -> Self {
        CustomerSummaryModuleAction::CustomerSummary(action)
    }
}
