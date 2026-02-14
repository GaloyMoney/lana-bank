use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub const CUSTOMER_SYNC: audit::SystemActor = audit::SystemActor::new("customer-sync");
pub const SUMSUB: audit::SystemActor = audit::SystemActor::new("sumsub");
pub use public_id::PublicId;

es_entity::entity_id! {
    CustomerId,
    CustomerDocumentId,
    ProspectId;

    CustomerId => document_storage::ReferenceId,
    CustomerId => public_id::PublicIdTargetId,
    CustomerDocumentId => document_storage::DocumentId,
    ProspectId => public_id::PublicIdTargetId,
    ProspectId => CustomerId
}

#[derive(Debug, Default, Deserialize, Clone, Copy, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum KycLevel {
    #[default]
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
pub enum KycVerification {
    #[default]
    PendingVerification,
    Verified,
    Rejected,
}

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    strum::Display,
    strum::EnumString,
    Default,
    sqlx::Type,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[sqlx(type_name = "text")]
pub enum Activity {
    #[default]
    Inactive,
    Active,
    Suspended,
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
pub enum KycStatus {
    #[default]
    NotStarted,
    Started,
    Pending,
    Approved,
    Declined,
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
pub enum ProspectStatus {
    #[default]
    Open,
    Converted,
    Closed,
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
pub enum ProspectStage {
    #[default]
    New,
    KycStarted,
    KycPending,
    KycDeclined,
    Converted,
    Closed,
}

impl KycVerification {
    pub fn is_verified(&self) -> bool {
        matches!(self, KycVerification::Verified)
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
pub type ProspectAllOrOne = AllOrOne<ProspectId>;

permission_sets_macro::permission_sets! {
    CustomerViewer("Can view customer profiles and account details"),
    CustomerWriter("Can create, update, and manage customers, including initiating and approving KYC processes"),
}

pub const CUSTOMER_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("customer");
pub const PROSPECT_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("prospect");

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CustomerObject {
    Customer(CustomerAllOrOne),
    CustomerDocument(CustomerDocumentAllOrOne),
    Prospect(ProspectAllOrOne),
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
    pub fn all_prospects() -> CustomerObject {
        CustomerObject::Prospect(AllOrOne::All)
    }
    pub fn prospect(id: impl Into<Option<ProspectId>>) -> CustomerObject {
        match id.into() {
            Some(id) => CustomerObject::Prospect(AllOrOne::ById(id)),
            None => CustomerObject::all_prospects(),
        }
    }
}

impl Display for CustomerObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CustomerObjectDiscriminants::from(self);
        use CustomerObject::*;
        match self {
            Customer(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            CustomerDocument(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Prospect(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
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
            Prospect => {
                let obj_ref = id.parse().map_err(|_| "could not parse CustomerObject")?;
                CustomerObject::Prospect(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ProspectEntityAction {
    Read,
    Create,
    List,
    Update,
    Close,
    StartKyc,
    ApproveKyc,
    DeclineKyc,
}

impl ActionPermission for ProspectEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_CUSTOMER_VIEWER,
            Self::Create
            | Self::Update
            | Self::Close
            | Self::StartKyc
            | Self::ApproveKyc
            | Self::DeclineKyc => PERMISSION_SET_CUSTOMER_WRITER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustomerAction {
    Customer(CustomerEntityAction),
    CustomerDocument(CustomerDocumentEntityAction),
    Prospect(ProspectEntityAction),
}

impl CoreCustomerAction {
    pub const CUSTOMER_CREATE: Self = CoreCustomerAction::Customer(CustomerEntityAction::Create);
    pub const CUSTOMER_READ: Self = CoreCustomerAction::Customer(CustomerEntityAction::Read);
    pub const CUSTOMER_LIST: Self = CoreCustomerAction::Customer(CustomerEntityAction::List);
    pub const CUSTOMER_UPDATE: Self = CoreCustomerAction::Customer(CustomerEntityAction::Update);
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
    pub const PROSPECT_CREATE: Self = CoreCustomerAction::Prospect(ProspectEntityAction::Create);
    pub const PROSPECT_READ: Self = CoreCustomerAction::Prospect(ProspectEntityAction::Read);
    pub const PROSPECT_LIST: Self = CoreCustomerAction::Prospect(ProspectEntityAction::List);
    pub const PROSPECT_UPDATE: Self = CoreCustomerAction::Prospect(ProspectEntityAction::Update);
    pub const PROSPECT_CLOSE: Self = CoreCustomerAction::Prospect(ProspectEntityAction::Close);
    pub const PROSPECT_START_KYC: Self =
        CoreCustomerAction::Prospect(ProspectEntityAction::StartKyc);
    pub const PROSPECT_APPROVE_KYC: Self =
        CoreCustomerAction::Prospect(ProspectEntityAction::ApproveKyc);
    pub const PROSPECT_DECLINE_KYC: Self =
        CoreCustomerAction::Prospect(ProspectEntityAction::DeclineKyc);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCustomerActionDiscriminants::*;
        use strum::VariantArray;

        CoreCustomerActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Customer => map_action!(customer, Customer, CustomerEntityAction),
                CustomerDocument => {
                    map_action!(customer, CustomerDocument, CustomerDocumentEntityAction)
                }
                Prospect => map_action!(customer, Prospect, ProspectEntityAction),
            })
            .collect()
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CustomerEntityAction {
    Read,
    Create,
    List,
    Update,
    StartKyc,
    ApproveKyc,
    DeclineKyc,
}

impl ActionPermission for CustomerEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_CUSTOMER_VIEWER,

            // All other operations use WRITER permission
            Self::Create | Self::Update | Self::StartKyc | Self::ApproveKyc | Self::DeclineKyc => {
                PERMISSION_SET_CUSTOMER_WRITER
            }
        }
    }
}

impl Display for CoreCustomerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCustomerActionDiscriminants::from(self))?;
        use CoreCustomerAction::*;
        match self {
            Customer(action) => action.fmt(f),
            CustomerDocument(action) => action.fmt(f),
            Prospect(action) => action.fmt(f),
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
            Prospect => CoreCustomerAction::from(action.parse::<ProspectEntityAction>()?),
        };
        Ok(res)
    }
}

impl From<CustomerEntityAction> for CoreCustomerAction {
    fn from(action: CustomerEntityAction) -> Self {
        CoreCustomerAction::Customer(action)
    }
}

impl From<ProspectEntityAction> for CoreCustomerAction {
    fn from(action: ProspectEntityAction) -> Self {
        CoreCustomerAction::Prospect(action)
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

impl ActionPermission for CustomerDocumentEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List | Self::GenerateDownloadLink => PERMISSION_SET_CUSTOMER_VIEWER,

            Self::Create | Self::Delete => PERMISSION_SET_CUSTOMER_WRITER,
        }
    }
}

impl From<CustomerDocumentEntityAction> for CoreCustomerAction {
    fn from(action: CustomerDocumentEntityAction) -> Self {
        CoreCustomerAction::CustomerDocument(action)
    }
}
