#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use authz::{ActionPermission, AllOrOne};

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisbursalObject {
    Disbursal(DisbursalAllOrOne),
}

impl DisbursalObject {
    pub fn disbursal(id: DisbursalId) -> Self {
        DisbursalObject::Disbursal(authz::AllOrOne::ById(id))
    }

    pub fn all_disbursals() -> Self {
        DisbursalObject::Disbursal(authz::AllOrOne::All)
    }
}

impl std::fmt::Display for DisbursalObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisbursalObject::Disbursal(obj_ref) => write!(f, "disbursal/{obj_ref}"),
        }
    }
}

impl std::str::FromStr for DisbursalObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').ok_or("missing slash")?;
        match entity {
            "disbursal" => {
                let obj_ref = id.parse().map_err(|_| "could not parse DisbursalObject")?;
                Ok(DisbursalObject::Disbursal(obj_ref))
            }
            _ => Err("unknown entity type for DisbursalObject"),
        }
    }
}
