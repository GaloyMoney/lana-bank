use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub const OBLIGATION_SYNC: audit::SystemActor = audit::SystemActor::new("obligation-sync");

pub use cala_ledger::primitives::{AccountId as CalaAccountId, TransactionId as LedgerTxId};
pub use core_credit_terms::EffectiveDate;
pub use money::*;

es_entity::entity_id! {
    BeneficiaryId,
    PaymentId,
    PaymentAllocationId,
    ObligationId;

    ObligationId => job::JobId,
    PaymentId => LedgerTxId,
    PaymentAllocationId => LedgerTxId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObligationStatus {
    NotYetDue,
    Due,
    Overdue,
    Defaulted,
    Paid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum ObligationType {
    Disbursal,
    Interest,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ObligationsAmounts {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl std::ops::Add<ObligationsAmounts> for ObligationsAmounts {
    type Output = Self;

    fn add(self, other: ObligationsAmounts) -> Self {
        Self {
            disbursed: self.disbursed + other.disbursed,
            interest: self.interest + other.interest,
        }
    }
}

impl ObligationsAmounts {
    pub const ZERO: Self = Self {
        disbursed: UsdCents::ZERO,
        interest: UsdCents::ZERO,
    };

    pub fn total(&self) -> UsdCents {
        self.interest + self.disbursed
    }

    pub fn is_zero(&self) -> bool {
        self.disbursed.is_zero() && self.interest.is_zero()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PaymentDetailsForAllocation {
    pub payment_id: PaymentId,
    pub amount: UsdCents,
    pub beneficiary_id: BeneficiaryId,
    pub facility_payment_holding_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

#[derive(Debug, Clone, Copy)]
pub struct PaymentSourceAccountId(CalaAccountId);

// Note: DO NOT implement `From<CalaAccountId> for PaymentSourceAccountId` since
//       we want to avoid trivially passing any CalaAccountId into a place that
//       expects PaymentSourceAccountId.

impl From<PaymentSourceAccountId> for CalaAccountId {
    fn from(account_id: PaymentSourceAccountId) -> Self {
        account_id.0
    }
}

impl PaymentSourceAccountId {
    pub const fn new(account_id: CalaAccountId) -> Self {
        Self(account_id)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ObligationReceivableAccountIds {
    pub not_yet_due: CalaAccountId,
    pub due: CalaAccountId,
    pub overdue: CalaAccountId,
}

impl ObligationReceivableAccountIds {
    #[allow(clippy::new_without_default)]
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            not_yet_due: CalaAccountId::new(),
            due: CalaAccountId::new(),
            overdue: CalaAccountId::new(),
        }
    }

    pub fn id_for_status(&self, status: ObligationStatus) -> Option<CalaAccountId> {
        match status {
            ObligationStatus::NotYetDue => Some(self.not_yet_due),
            ObligationStatus::Due => Some(self.due),
            ObligationStatus::Overdue | ObligationStatus::Defaulted => Some(self.overdue),
            ObligationStatus::Paid => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum BalanceUpdatedSource {
    Obligation(ObligationId),
    PaymentAllocation(PaymentAllocationId),
}

impl From<ObligationId> for BalanceUpdatedSource {
    fn from(obligation_id: ObligationId) -> Self {
        Self::Obligation(obligation_id)
    }
}

impl From<PaymentAllocationId> for BalanceUpdatedSource {
    fn from(allocation_id: PaymentAllocationId) -> Self {
        Self::PaymentAllocation(allocation_id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BalanceUpdateData {
    pub source_id: BalanceUpdatedSource,
    pub ledger_tx_id: LedgerTxId,
    pub balance_type: ObligationType,
    pub amount: UsdCents,
    pub updated_at: DateTime<Utc>,
}

pub type ObligationAllOrOne = AllOrOne<ObligationId>;

pub const PERMISSION_SET_COLLECTION_WRITER: &str = "collection_writer";
pub const PERMISSION_SET_COLLECTION_VIEWER: &str = "collection_viewer";
pub const PERMISSION_SET_COLLECTION_PAYMENT_DATE: &str = "collection_payment_date";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollectionObject {
    Obligation(ObligationAllOrOne),
}

impl CoreCreditCollectionObject {
    pub fn obligation(id: ObligationId) -> Self {
        CoreCreditCollectionObject::Obligation(AllOrOne::ById(id))
    }

    pub fn all_obligations() -> Self {
        CoreCreditCollectionObject::Obligation(AllOrOne::All)
    }
}

impl std::fmt::Display for CoreCreditCollectionObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCreditCollectionObjectDiscriminants::from(self);
        use CoreCreditCollectionObject::*;
        match self {
            Obligation(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreCreditCollectionObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCreditCollectionObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Obligation => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCreditCollectionObject")?;
                CoreCreditCollectionObject::Obligation(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollectionAction {
    Obligation(ObligationAction),
}

impl CoreCreditCollectionAction {
    pub const OBLIGATION_READ: Self =
        CoreCreditCollectionAction::Obligation(ObligationAction::Read);
    pub const OBLIGATION_UPDATE_STATUS: Self =
        CoreCreditCollectionAction::Obligation(ObligationAction::UpdateStatus);
    pub const OBLIGATION_RECORD_PAYMENT: Self =
        CoreCreditCollectionAction::Obligation(ObligationAction::RecordPaymentAllocation);
    pub const OBLIGATION_RECORD_PAYMENT_WITH_DATE: Self =
        CoreCreditCollectionAction::Obligation(ObligationAction::RecordPaymentAllocationWithDate);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCreditCollectionActionDiscriminants::*;
        use strum::VariantArray;

        CoreCreditCollectionActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Obligation => map_action!(credit_collection, Obligation, ObligationAction),
            })
            .collect()
    }
}

impl std::fmt::Display for CoreCreditCollectionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:",
            CoreCreditCollectionActionDiscriminants::from(self)
        )?;
        use CoreCreditCollectionAction::*;
        match self {
            Obligation(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreCreditCollectionAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreCreditCollectionActionDiscriminants::*;
        let res = match entity.parse()? {
            Obligation => CoreCreditCollectionAction::from(action.parse::<ObligationAction>()?),
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum ObligationAction {
    Read,
    UpdateStatus,
    RecordPaymentAllocation,
    RecordPaymentAllocationWithDate,
}

impl ActionPermission for ObligationAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_COLLECTION_VIEWER,
            Self::UpdateStatus | Self::RecordPaymentAllocation => PERMISSION_SET_COLLECTION_WRITER,
            Self::RecordPaymentAllocationWithDate => PERMISSION_SET_COLLECTION_PAYMENT_DATE,
        }
    }
}

impl From<ObligationAction> for CoreCreditCollectionAction {
    fn from(action: ObligationAction) -> Self {
        Self::Obligation(action)
    }
}
