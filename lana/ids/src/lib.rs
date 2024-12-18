es_entity::entity_id! {
    CustomerId,
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
    CustomerId => deposit::DepositAccountHolderId,
}
