use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use std::fmt;

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};
pub use core_custody::{CustodianId, WalletId as CustodyWalletId};
pub use core_customer::{CustomerId, CustomerType};
pub use core_money::*;
pub use credit_terms::TermValues;
use credit_terms::balance_summary::CreditFacilityBalanceSummary;
pub use governance::ApprovalProcessId;

es_entity::entity_id! {
    CreditFacilityProposalId,
    PendingCreditFacilityId,
    CreditFacilityId,
    CollateralId;

    CreditFacilityProposalId => PendingCreditFacilityId,

    CreditFacilityProposalId => CreditFacilityId,
    PendingCreditFacilityId => CreditFacilityId,

    CreditFacilityId => governance::ApprovalProcessId,
    CreditFacilityProposalId => governance::ApprovalProcessId,

    CreditFacilityId => job::JobId,

    CreditFacilityId => public_id::PublicIdTargetId,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CreditFacilityReceivable {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl From<CreditFacilityBalanceSummary> for CreditFacilityReceivable {
    fn from(balance: CreditFacilityBalanceSummary) -> Self {
        Self {
            disbursed: balance.disbursed_outstanding_payable(),
            interest: balance.interest_outstanding_payable(),
        }
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CreditFacilityProposalStatus {
    #[default]
    PendingCustomerApproval,
    CustomerDenied,
    PendingApproval,
    Approved,
    Denied,
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

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum PendingCreditFacilityCollateralizationState {
    FullyCollateralized,
    #[default]
    UnderCollateralized,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum PendingCreditFacilityStatus {
    #[default]
    PendingCollateralization,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CollateralizationRatio {
    Finite(Decimal),
    Infinite,
}

impl Default for CollateralizationRatio {
    fn default() -> Self {
        Self::Finite(Decimal::ZERO)
    }
}
