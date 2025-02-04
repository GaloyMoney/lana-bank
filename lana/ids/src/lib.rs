use cala_ledger::primitives::TransactionId as LedgerTransactionId;

es_entity::entity_id! {
    DocumentId,
    CreditFacilityId,
    DisbursalId,
    InterestAccrualId,
    TermsTemplateId,
    ReportId;

    CreditFacilityId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    ReportId => job::JobId,
    CreditFacilityId => job::JobId,
    InterestAccrualId => job::JobId,

    DisbursalId => LedgerTransactionId,
}
