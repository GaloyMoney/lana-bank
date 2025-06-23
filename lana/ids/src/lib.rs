use cala_ledger::primitives::TransactionId as LedgerTransactionId;

es_entity::entity_id! {
    CreditFacilityId,
    DisbursalId,
    PaymentId,
    InterestAccrualCycleId,
    TermsTemplateId,
    ReportId;

    CreditFacilityId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    ReportId => job::JobId,
    CreditFacilityId => job::JobId,
    InterestAccrualCycleId => job::JobId,

    DisbursalId => LedgerTransactionId,
    PaymentId => LedgerTransactionId,
}
