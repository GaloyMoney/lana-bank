use serde::{Deserialize, Serialize};

use credit_terms::balance_summary::CreditFacilityBalanceSummary;

pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};
pub use core_customer::{CustomerId, CustomerType};
pub use core_money::UsdCents;
pub use credit_terms::TermValues;
pub use governance::ApprovalProcessId;

es_entity::entity_id! {
    CreditFacilityProposalId,
    PendingCreditFacilityId,
    CreditFacilityId;

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
