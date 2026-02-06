#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub use cala_ledger::primitives::{AccountId as CalaAccountId, TransactionId as LedgerTxId};
pub use core_credit_collection::{
    BeneficiaryId, NewObligation, ObligationId, ObligationReceivableAccountIds, ObligationType,
};
pub use core_credit_terms::EffectiveDate;
pub use core_money::UsdCents;
pub use governance::ApprovalProcessId;
pub use public_id::PublicId;

es_entity::entity_id! {
    DisbursalId;

    DisbursalId => governance::ApprovalProcessId,
    DisbursalId => LedgerTxId,
    DisbursalId => public_id::PublicIdTargetId,
}

pub type DisbursalAllOrOne = AllOrOne<DisbursalId>;

pub const DISBURSAL_REF_TARGET: public_id::PublicIdTargetType =
    public_id::PublicIdTargetType::new("disbursal");

pub const DISBURSAL_TRANSACTION_ENTITY_TYPE: core_accounting::EntityType =
    core_accounting::EntityType::new("Disbursal");

pub const PERMISSION_SET_DISBURSAL_VIEWER: &str = "disbursal_viewer";
pub const PERMISSION_SET_DISBURSAL_WRITER: &str = "disbursal_writer";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum DisbursalStatus {
    New,
    Approved,
    Denied,
    Confirmed,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DisbursalAction {
    Initiate,
    Settle,
    List,
    Read,
}

impl ActionPermission for DisbursalAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List | Self::Read => PERMISSION_SET_DISBURSAL_VIEWER,
            Self::Initiate | Self::Settle => PERMISSION_SET_DISBURSAL_WRITER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditDisbursalObject {
    Disbursal(DisbursalAllOrOne),
}

impl CoreCreditDisbursalObject {
    pub fn disbursal(id: DisbursalId) -> Self {
        CoreCreditDisbursalObject::Disbursal(AllOrOne::ById(id))
    }

    pub fn all_disbursals() -> Self {
        CoreCreditDisbursalObject::Disbursal(AllOrOne::All)
    }
}

impl std::fmt::Display for CoreCreditDisbursalObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCreditDisbursalObjectDiscriminants::from(self);
        use CoreCreditDisbursalObject::*;
        match self {
            Disbursal(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl std::str::FromStr for CoreCreditDisbursalObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCreditDisbursalObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Disbursal => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCreditDisbursalObject")?;
                CoreCreditDisbursalObject::Disbursal(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditDisbursalAction {
    Disbursal(DisbursalAction),
}

impl CoreCreditDisbursalAction {
    pub const DISBURSAL_INITIATE: Self =
        CoreCreditDisbursalAction::Disbursal(DisbursalAction::Initiate);
    pub const DISBURSAL_SETTLE: Self =
        CoreCreditDisbursalAction::Disbursal(DisbursalAction::Settle);
    pub const DISBURSAL_LIST: Self = CoreCreditDisbursalAction::Disbursal(DisbursalAction::List);
    pub const DISBURSAL_READ: Self = CoreCreditDisbursalAction::Disbursal(DisbursalAction::Read);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCreditDisbursalActionDiscriminants::*;
        use strum::VariantArray;

        CoreCreditDisbursalActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Disbursal => map_action!(credit_disbursal, Disbursal, DisbursalAction),
            })
            .collect()
    }
}

impl From<DisbursalAction> for CoreCreditDisbursalAction {
    fn from(action: DisbursalAction) -> Self {
        CoreCreditDisbursalAction::Disbursal(action)
    }
}

impl std::fmt::Display for CoreCreditDisbursalAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCreditDisbursalActionDiscriminants::from(self))?;
        use CoreCreditDisbursalAction::*;
        match self {
            Disbursal(action) => action.fmt(f),
        }
    }
}

impl std::str::FromStr for CoreCreditDisbursalAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreCreditDisbursalActionDiscriminants::*;
        let res = match entity.parse()? {
            Disbursal => CoreCreditDisbursalAction::from(action.parse::<DisbursalAction>()?),
        };
        Ok(res)
    }
}
