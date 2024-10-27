#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

es_entity::entity_id! { UserId }
es_entity::entity_id! { CommitteeId }
es_entity::entity_id! { CustomerId }
es_entity::entity_id! { LineOfCreditContractId }
es_entity::entity_id! { WithdrawId }
es_entity::entity_id! { DepositId }
es_entity::entity_id! { DocumentId }
es_entity::entity_id! { LoanId }
es_entity::entity_id! { CreditFacilityId }
es_entity::entity_id! { DisbursementId }
es_entity::entity_id! { InterestAccrualId }
es_entity::entity_id! { TermsTemplateId }
es_entity::entity_id! { ReportId }

pub use job::JobId;

impl From<LoanId> for JobId {
    fn from(id: LoanId) -> Self {
        JobId::from(id.0)
    }
}
impl From<CreditFacilityId> for JobId {
    fn from(id: CreditFacilityId) -> Self {
        JobId::from(id.0)
    }
}
impl From<ReportId> for JobId {
    fn from(id: ReportId) -> Self {
        JobId::from(id.0)
    }
}
