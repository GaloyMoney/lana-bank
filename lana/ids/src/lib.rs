use cala_ledger::primitives::{
    AccountSetId as LedgerAccountSetId, TransactionId as LedgerTransactionId,
};

es_entity::entity_id! {
    DocumentId,
    CreditFacilityId,
    DisbursalId,
    InterestAccrualId,
    TermsTemplateId,
    TrialBalanceId,
    ProfitAndLossStatementId,
    ReportId;

    CreditFacilityId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    ReportId => job::JobId,
    CreditFacilityId => job::JobId,
    InterestAccrualId => job::JobId,

    DisbursalId => LedgerTransactionId,
    // CustomerId => deposit::DepositAccountHolderId,
    TrialBalanceId => LedgerAccountSetId,
    ProfitAndLossStatementId => LedgerAccountSetId,
}
