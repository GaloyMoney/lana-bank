use serde::{Deserialize, Serialize};

es_entity::entity_id! {
    LoanAgreementId,
}

#[derive(Debug, Clone)]
pub struct GeneratedLoanAgreementDownloadLink {
    pub loan_agreement_id: LoanAgreementId,
    pub link: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
pub enum LoanAgreementStatus {
    Pending,
    Completed,
    Failed,
}