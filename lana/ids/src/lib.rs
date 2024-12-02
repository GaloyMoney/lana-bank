es_entity::entity_id! {
    CustomerId,
    WithdrawalId,
    DepositId,
    DocumentId,
    CreditFacilityId,
    DisbursalId,
    InterestAccrualId,
    TermsTemplateId,
    ReportId;

    WithdrawalId => governance::ApprovalProcessId,
    CreditFacilityId => governance::ApprovalProcessId,
    DisbursalId => governance::ApprovalProcessId,

    ReportId => job::JobId,
    CreditFacilityId => job::JobId,
    InterestAccrualId => job::JobId,
}
