use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub const CUSTODY_KEY_ROTATION: audit::SystemActor =
    audit::SystemActor::new("custody-key-rotation");
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

es_entity::entity_id! {
    CustodianId,
    WalletId
}

#[cfg(feature = "mock-custodian")]
impl CustodianId {
    pub const fn is_mock_custodian(self) -> bool {
        self.0.is_nil()
    }

    pub const fn mock_custodian_id() -> Self {
        Self(uuid::Uuid::nil())
    }
}

#[derive(Debug, Clone)]
pub struct ExternalWallet {
    pub external_id: String,
    pub address: String,
    pub network: WalletNetwork,
    pub full_response: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum WalletNetwork {
    Testnet3,
    Testnet4,
    Mainnet,
}

permission_sets_macro::permission_sets! {
    CustodyViewer("Can view custodian configurations"),
    CustodyWriter("Can create and manage custodian configurations"),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyAction {
    Custodian(CustodianAction),
    Wallet(WalletAction),
}

impl CoreCustodyAction {
    pub const CUSTODIAN_CREATE: Self = CoreCustodyAction::Custodian(CustodianAction::Create);
    pub const CUSTODIAN_LIST: Self = CoreCustodyAction::Custodian(CustodianAction::List);
    pub const CUSTODIAN_UPDATE: Self = CoreCustodyAction::Custodian(CustodianAction::Update);
    pub const CUSTODIAN_CREATE_WALLET: Self =
        CoreCustodyAction::Custodian(CustodianAction::CreateWallet);

    pub const WALLET_UPDATE: Self = CoreCustodyAction::Wallet(WalletAction::Update);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCustodyActionDiscriminants::*;
        use strum::VariantArray;

        CoreCustodyActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Custodian => map_action!(custody, Custodian, CustodianAction),
                Wallet => map_action!(custody, Wallet, WalletAction),
            })
            .collect()
    }
}

impl core::fmt::Display for CoreCustodyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCustodyActionDiscriminants::from(self))?;
        match self {
            Self::Custodian(action) => action.fmt(f),
            Self::Wallet(action) => action.fmt(f),
        }
    }
}

impl core::str::FromStr for CoreCustodyAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreCustodyActionDiscriminants::*;
        let res = match entity.parse()? {
            Custodian => CoreCustodyAction::from(action.parse::<CustodianAction>()?),
            Wallet => CoreCustodyAction::from(action.parse::<WalletAction>()?),
        };

        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CustodianAction {
    Create,
    List,
    Update,
    CreateWallet,
}

impl ActionPermission for CustodianAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Create | Self::CreateWallet | Self::Update => PERMISSION_SET_CUSTODY_WRITER,
            Self::List => PERMISSION_SET_CUSTODY_VIEWER,
        }
    }
}

impl From<CustodianAction> for CoreCustodyAction {
    fn from(action: CustodianAction) -> Self {
        Self::Custodian(action)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum WalletAction {
    Update,
}

impl ActionPermission for WalletAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Update => PERMISSION_SET_CUSTODY_WRITER,
        }
    }
}

impl From<WalletAction> for CoreCustodyAction {
    fn from(action: WalletAction) -> Self {
        Self::Wallet(action)
    }
}

pub type CustodianAllOrOne = AllOrOne<CustodianId>;
pub type WalletAllOrOne = AllOrOne<WalletId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyObject {
    Custodian(CustodianAllOrOne),
    Wallet(WalletAllOrOne),
}

impl CoreCustodyObject {
    pub const fn all_custodians() -> Self {
        CoreCustodyObject::Custodian(AllOrOne::All)
    }

    pub const fn custodian(id: CustodianId) -> Self {
        CoreCustodyObject::Custodian(AllOrOne::ById(id))
    }

    pub const fn wallet(id: WalletId) -> Self {
        CoreCustodyObject::Wallet(AllOrOne::ById(id))
    }
}

impl core::fmt::Display for CoreCustodyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCustodyObjectDiscriminants::from(self);
        match self {
            Self::Custodian(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Self::Wallet(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl core::str::FromStr for CoreCustodyObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCustodyObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Custodian => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCustodyObject")?;
                Self::Custodian(obj_ref)
            }
            Wallet => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCustodyObject")?;
                Self::Wallet(obj_ref)
            }
        };
        Ok(res)
    }
}
