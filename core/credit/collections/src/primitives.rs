use std::str::FromStr;

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};
pub use core_money::*;

es_entity::entity_id! {
    BeneficiaryId,
    PaymentId,
    PaymentAllocationId,
    ObligationId;

    ObligationId => job::JobId,
    PaymentId => LedgerTxId,
    PaymentAllocationId => LedgerTxId,
}

pub type ObligationAllOrOne = AllOrOne<ObligationId>;

pub const PERMISSION_SET_COLLECTIONS_WRITER: &str = "credit_writer";
pub const PERMISSION_SET_COLLECTIONS_VIEWER: &str = "credit_viewer";
pub const PERMISSION_SET_COLLECTIONS_PAYMENT_DATE: &str = "credit_payment_date";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollectionsObject {
    Obligation(ObligationAllOrOne),
}

impl CoreCreditCollectionsObject {
    pub fn obligation(id: ObligationId) -> Self {
        CoreCreditCollectionsObject::Obligation(AllOrOne::ById(id))
    }

    pub fn all_obligations() -> Self {
        CoreCreditCollectionsObject::Obligation(AllOrOne::All)
    }
}

impl std::fmt::Display for CoreCreditCollectionsObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCreditCollectionsObjectDiscriminants::from(self);
        use CoreCreditCollectionsObject::*;
        match self {
            Obligation(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreCreditCollectionsObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCreditCollectionsObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Obligation => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCreditCollectionsObject")?;
                CoreCreditCollectionsObject::Obligation(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditCollectionsAction {
    Obligation(ObligationAction),
}

impl CoreCreditCollectionsAction {
    pub const OBLIGATION_READ: Self =
        CoreCreditCollectionsAction::Obligation(ObligationAction::Read);
    pub const OBLIGATION_UPDATE_STATUS: Self =
        CoreCreditCollectionsAction::Obligation(ObligationAction::UpdateStatus);
    pub const OBLIGATION_RECORD_PAYMENT: Self =
        CoreCreditCollectionsAction::Obligation(ObligationAction::RecordPaymentAllocation);
    pub const OBLIGATION_RECORD_PAYMENT_WITH_DATE: Self =
        CoreCreditCollectionsAction::Obligation(ObligationAction::RecordPaymentAllocationWithDate);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCreditCollectionsActionDiscriminants::*;
        use strum::VariantArray;

        CoreCreditCollectionsActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                Obligation => map_action!(collections, Obligation, ObligationAction),
            })
            .collect()
    }
}

impl std::fmt::Display for CoreCreditCollectionsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:",
            CoreCreditCollectionsActionDiscriminants::from(self)
        )?;
        use CoreCreditCollectionsAction::*;
        match self {
            Obligation(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreCreditCollectionsAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreCreditCollectionsActionDiscriminants::*;
        let res = match entity.parse()? {
            Obligation => CoreCreditCollectionsAction::from(action.parse::<ObligationAction>()?),
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
            Self::Read => PERMISSION_SET_COLLECTIONS_VIEWER,
            Self::UpdateStatus | Self::RecordPaymentAllocation => PERMISSION_SET_COLLECTIONS_WRITER,
            Self::RecordPaymentAllocationWithDate => PERMISSION_SET_COLLECTIONS_PAYMENT_DATE,
        }
    }
}

impl From<ObligationAction> for CoreCreditCollectionsAction {
    fn from(action: ObligationAction) -> Self {
        Self::Obligation(action)
    }
}
