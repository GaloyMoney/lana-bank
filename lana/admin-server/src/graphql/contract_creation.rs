use async_graphql::*;

use crate::primitives::*;

#[derive(async_graphql::Enum, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LoanAgreementStatus {
    Pending,
    Completed,
    Failed,
}

impl From<lana_app::contract_creation::LoanAgreementStatus> for LoanAgreementStatus {
    fn from(status: lana_app::contract_creation::LoanAgreementStatus) -> Self {
        match status {
            lana_app::contract_creation::LoanAgreementStatus::Pending => Self::Pending,
            lana_app::contract_creation::LoanAgreementStatus::Completed => Self::Completed,
            lana_app::contract_creation::LoanAgreementStatus::Failed => Self::Failed,
            lana_app::contract_creation::LoanAgreementStatus::Removed => Self::Failed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct LoanAgreement {
    id: ID,
    loan_agreement_id: UUID,
    status: LoanAgreementStatus,
    created_at: Timestamp,
}

impl LoanAgreement {
    pub fn new(
        id: uuid::Uuid,
        status: LoanAgreementStatus,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id: ID::from(format!("LoanAgreement:{}", id)),
            loan_agreement_id: UUID::from(id),
            status,
            created_at: created_at.into(),
        }
    }
}

impl From<lana_app::contract_creation::LoanAgreement> for LoanAgreement {
    fn from(domain_loan_agreement: lana_app::contract_creation::LoanAgreement) -> Self {
        Self::new(
            domain_loan_agreement.id,
            domain_loan_agreement.status.into(),
            domain_loan_agreement.created_at,
        )
    }
}

#[derive(InputObject)]
pub struct LoanAgreementGenerateInput {
    pub customer_id: UUID,
}

crate::mutation_payload! { LoanAgreementGeneratePayload, loan_agreement: LoanAgreement }

#[derive(InputObject)]
pub struct LoanAgreementDownloadLinksGenerateInput {
    pub loan_agreement_id: UUID,
}

#[derive(SimpleObject)]
pub struct LoanAgreementDownloadLinksGeneratePayload {
    pub loan_agreement_id: UUID,
    pub link: String,
}

impl From<lana_app::document::GeneratedDocumentDownloadLink>
    for LoanAgreementDownloadLinksGeneratePayload
{
    fn from(value: lana_app::document::GeneratedDocumentDownloadLink) -> Self {
        Self {
            loan_agreement_id: UUID::from(value.document_id),
            link: value.link,
        }
    }
}
