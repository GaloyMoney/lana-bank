use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::AllOrOne;

es_entity::entity_id! {
    CustomerId;
    CustomerId => deposit::DepositAccountHolderId,
}

es_entity::entity_id! { AuthenticationId }

#[derive(Debug, Deserialize, Clone, Copy, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
pub enum KycLevel {
    NotKyced,
    Basic,
    Advanced,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
pub enum AccountStatus {
    #[default]
    Inactive,
    Active,
}

pub type CustomerAllOrOne = AllOrOne<CustomerId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CustomerObject {
    Customer(CustomerAllOrOne),
}

impl CustomerObject {
    pub fn all_customers() -> CustomerObject {
        CustomerObject::Customer(AllOrOne::All)
    }
    pub fn customer(id: impl Into<Option<CustomerId>>) -> CustomerObject {
        match id.into() {
            Some(id) => CustomerObject::Customer(AllOrOne::ById(id)),
            None => CustomerObject::all_customers(),
        }
    }
}

impl Display for CustomerObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CustomerObjectDiscriminants::from(self);
        use CustomerObject::*;
        match self {
            Customer(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CustomerObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CustomerObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Customer => {
                let obj_ref = id.parse().map_err(|_| "could not parse CustomerObject")?;
                CustomerObject::Customer(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustomerAction {
    Customer(CustomerEntityAction),
}

impl CoreCustomerAction {
    pub const CUSTOMER_CREATE: Self = CoreCustomerAction::Customer(CustomerEntityAction::Create);
    pub const CUSTOMER_READ: Self = CoreCustomerAction::Customer(CustomerEntityAction::Read);
    pub const CUSTOMER_LIST: Self = CoreCustomerAction::Customer(CustomerEntityAction::List);
    pub const CUSTOMER_UPDATE: Self = CoreCustomerAction::Customer(CustomerEntityAction::Update);
    pub const CUSTOMER_UPDATE_AUTHENTICATION_ID: Self =
        CoreCustomerAction::Customer(CustomerEntityAction::UpdateAuthenticationId);
    pub const CUSTOMER_START_KYC: Self =
        CoreCustomerAction::Customer(CustomerEntityAction::StartKyc);
    pub const CUSTOMER_APPROVE_KYC: Self =
        CoreCustomerAction::Customer(CustomerEntityAction::ApproveKyc);
    pub const CUSTOMER_DECLINE_KYC: Self =
        CoreCustomerAction::Customer(CustomerEntityAction::DeclineKyc);
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CustomerEntityAction {
    Read,
    Create,
    List,
    Update,
    UpdateAuthenticationId,
    StartKyc,
    ApproveKyc,
    DeclineKyc,
}

impl Display for CoreCustomerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCustomerActionDiscriminants::from(self))?;
        use CoreCustomerAction::*;
        match self {
            Customer(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreCustomerAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreCustomerActionDiscriminants::*;
        let res = match entity.parse()? {
            Customer => CoreCustomerAction::from(action.parse::<CustomerEntityAction>()?),
        };
        Ok(res)
    }
}

impl From<CustomerEntityAction> for CoreCustomerAction {
    fn from(action: CustomerEntityAction) -> Self {
        CoreCustomerAction::Customer(action)
    }
}
