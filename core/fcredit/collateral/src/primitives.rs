use serde::{Deserialize, Serialize};

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};

pub use core_custody::WalletId as CustodyWalletId;
pub use core_money::*;

es_entity::entity_id! {
    CollateralId,
    CreditFacilityId,
    PendingCreditFacilityId
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CollateralAction {
    Add,
    Remove,
}

pub struct CollateralUpdate {
    pub tx_id: LedgerTxId,
    pub abs_diff: Satoshis,
    pub action: CollateralAction,
    pub effective: chrono::NaiveDate,
}
