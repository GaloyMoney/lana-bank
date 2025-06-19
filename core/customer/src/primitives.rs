use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{action_description::*, AllOrOne};

es_entity::entity_id! {
    CustomerId,
    CustomerDocumentId;

    CustomerId => document_storage::ReferenceId,
    CustomerDocumentId => document_storage::DocumentId
}

es_entity::entity_id! { AuthenticationId }

#[derive(Debug, Deserialize, Clone, Copy, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum KycLevel {
    NotKyced,
    Basic,
    Advanced,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum AccountStatus {
    #[default]
    Inactive,
    Active,
}

#[derive(Debug, Deserialize, Clone, Copy, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum CustomerType {
    Individual,
    GovernmentEntity,
    PrivateCompany,
    Bank,
    FinancialInstitution,
    ForeignAgencyOrSubsidiary,
    NonDomiciledCompany,
}

impl Display for CustomerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomerType::Individual => write!(f, "Individual"),
            CustomerType::GovernmentEntity => write!(f, "Government Entity"),
            CustomerType::PrivateCompany => write!(f, "Private Company"),
            CustomerType::Bank => write!(f, "Bank"),
            CustomerType::FinancialInstitution => write!(f, "Financial Institution"),
            CustomerType::ForeignAgencyOrSubsidiary => write!(f, "Foreign Agency or Subsidiary"),
            CustomerType::NonDomiciledCompany => write!(f, "Non-Domiciled Company"),
        }
    }
}

impl AccountStatus {
    pub fn is_inactive(&self) -> bool {
        matches!(self, AccountStatus::Inactive)
    }
}

impl From<CustomerType> for String {
    fn from(customer_type: CustomerType) -> Self {
        match customer_type {
            CustomerType::Individual => "Individual".to_string(),
            CustomerType::GovernmentEntity => "Government Entity".to_string(),
            CustomerType::PrivateCompany => "Private Company".to_string(),
            CustomerType::Bank => "Bank".to_string(),
            CustomerType::FinancialInstitution => "Financial Institution".to_string(),
            CustomerType::ForeignAgencyOrSubsidiary => "Foreign Agency or Subsidiary".to_string(),
            CustomerType::NonDomiciledCompany => "Non-Domiciled Company".to_string(),
        }
    }
}

pub type CustomerAllOrOne = AllOrOne<CustomerId>;
pub type CustomerDocumentAllOrOne = AllOrOne<CustomerDocumentId>;

pub const PERMISSION_SET_CUSTOMER_VIEWER: &str = "customer_viewer";
pub const PERMISSION_SET_CUSTOMER_WRITER: &str = "customer_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CustomerObject {
    Customer(CustomerAllOrOne),
    CustomerDocument(CustomerDocumentAllOrOne),
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
    pub fn all_customer_documents() -> CustomerObject {
        CustomerObject::CustomerDocument(AllOrOne::All)
    }
    pub fn customer_document(id: impl Into<Option<CustomerDocumentId>>) -> CustomerObject {
        match id.into() {
            Some(id) => CustomerObject::CustomerDocument(AllOrOne::ById(id)),
            None => CustomerObject::all_customer_documents(),
        }
    }
}

impl Display for CustomerObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CustomerObjectDiscriminants::from(self);
        use CustomerObject::*;
        match self {
            Customer(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            CustomerDocument(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
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
            CustomerDocument => {
                let obj_ref = id.parse().map_err(|_| "could not parse CustomerObject")?;
                CustomerObject::CustomerDocument(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustomerAction {
    Customer(CustomerEntityAction),
    CustomerDocument(CustomerDocumentEntityAction),
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
    pub const CUSTOMER_DOCUMENT_CREATE: Self =
        CoreCustomerAction::CustomerDocument(CustomerDocumentEntityAction::Create);
    pub const CUSTOMER_DOCUMENT_READ: Self =
        CoreCustomerAction::CustomerDocument(CustomerDocumentEntityAction::Read);
    pub const CUSTOMER_DOCUMENT_LIST: Self =
        CoreCustomerAction::CustomerDocument(CustomerDocumentEntityAction::List);
    pub const CUSTOMER_DOCUMENT_DELETE: Self =
        CoreCustomerAction::CustomerDocument(CustomerDocumentEntityAction::Delete);
    pub const CUSTOMER_DOCUMENT_GENERATE_DOWNLOAD_LINK: Self =
        CoreCustomerAction::CustomerDocument(CustomerDocumentEntityAction::GenerateDownloadLink);

    pub fn entities() -> Vec<(
        CoreCustomerActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreCustomerActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreCustomerActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                Customer => CustomerEntityAction::describe(),
                CustomerDocument => CustomerDocumentEntityAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
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

impl CustomerEntityAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER]),

                Self::Read => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_CUSTOMER_VIEWER,
                        PERMISSION_SET_CUSTOMER_WRITER,
                    ],
                ),

                Self::List => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_CUSTOMER_WRITER,
                        PERMISSION_SET_CUSTOMER_VIEWER,
                    ],
                ),

                Self::Update => ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER]),

                Self::UpdateAuthenticationId => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER])
                }

                Self::StartKyc => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER])
                }

                Self::ApproveKyc => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER])
                }

                Self::DeclineKyc => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER])
                }
            };
            res.push(action_description);
        }

        res
    }
}

impl Display for CoreCustomerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCustomerActionDiscriminants::from(self))?;
        use CoreCustomerAction::*;
        match self {
            Customer(action) => action.fmt(f),
            CustomerDocument(action) => action.fmt(f),
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
            CustomerDocument => {
                CoreCustomerAction::from(action.parse::<CustomerDocumentEntityAction>()?)
            }
        };
        Ok(res)
    }
}

impl From<CustomerEntityAction> for CoreCustomerAction {
    fn from(action: CustomerEntityAction) -> Self {
        CoreCustomerAction::Customer(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CustomerDocumentEntityAction {
    Read,
    Create,
    List,
    Delete,
    GenerateDownloadLink,
}

impl CustomerDocumentEntityAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER]),

                Self::Read => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_CUSTOMER_VIEWER,
                        PERMISSION_SET_CUSTOMER_WRITER,
                    ],
                ),

                Self::List => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_CUSTOMER_WRITER,
                        PERMISSION_SET_CUSTOMER_VIEWER,
                    ],
                ),

                Self::GenerateDownloadLink => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_CUSTOMER_VIEWER,
                        PERMISSION_SET_CUSTOMER_WRITER,
                    ],
                ),

                Self::Delete => ActionDescription::new(variant, &[PERMISSION_SET_CUSTOMER_WRITER]),
            };
            res.push(action_description);
        }

        res
    }
}

impl From<CustomerDocumentEntityAction> for CoreCustomerAction {
    fn from(action: CustomerDocumentEntityAction) -> Self {
        CoreCustomerAction::CustomerDocument(action)
    }
}
