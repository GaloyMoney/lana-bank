use cala_ledger::primitives::{
    AccountSetId as LedgerAccountSetId, TransactionId as LedgerTransactionId,
};

es_entity::entity_id! {
    CustomerId,
    DocumentId,
    CreditFacilityId,
    DisbursalId,
    InterestAccrualId,
    TermsTemplateId,
    TrialBalanceId,
    ReportId;

    CreditFacilityId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    ReportId => job::JobId,
    CreditFacilityId => job::JobId,
    InterestAccrualId => job::JobId,

    DisbursalId => LedgerTransactionId,
    CustomerId => deposit::DepositAccountHolderId,
    TrialBalanceId => LedgerAccountSetId,
}
