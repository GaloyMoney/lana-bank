use async_graphql::*;

use crate::primitives::*;

#[derive(async_graphql::Enum, Clone, Debug, PartialEq, Eq, Copy)]
pub enum CreditFacilityAgreementStatus {
    Pending,
    Completed,
    Failed,
}

impl From<lana_app::contract_creation::CreditFacilityAgreementStatus>
    for CreditFacilityAgreementStatus
{
    fn from(status: lana_app::contract_creation::CreditFacilityAgreementStatus) -> Self {
        match status {
            lana_app::contract_creation::CreditFacilityAgreementStatus::Pending => Self::Pending,
            lana_app::contract_creation::CreditFacilityAgreementStatus::Completed => {
                Self::Completed
            }
            lana_app::contract_creation::CreditFacilityAgreementStatus::Failed => Self::Failed,
            lana_app::contract_creation::CreditFacilityAgreementStatus::Removed => Self::Failed,
        }
    }
}

#[derive(SimpleObject, Clone)]
#[graphql(directive = crate::graphql::entity_key::entity_key::apply("creditFacilityAgreementId".to_string()))]
pub struct CreditFacilityAgreement {
    credit_facility_agreement_id: UUID,
    status: CreditFacilityAgreementStatus,
    created_at: Timestamp,
}

impl CreditFacilityAgreement {
    pub fn new(
        id: uuid::Uuid,
        status: CreditFacilityAgreementStatus,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            credit_facility_agreement_id: UUID::from(id),
            status,
            created_at: created_at.into(),
        }
    }
}

impl From<lana_app::contract_creation::CreditFacilityAgreement> for CreditFacilityAgreement {
    fn from(
        domain_credit_facility_agreement: lana_app::contract_creation::CreditFacilityAgreement,
    ) -> Self {
        Self::new(
            domain_credit_facility_agreement.id,
            domain_credit_facility_agreement.status.into(),
            domain_credit_facility_agreement.created_at,
        )
    }
}

#[derive(InputObject)]
pub struct CreditFacilityAgreementGenerateInput {
    pub credit_facility_id: UUID,
}

crate::mutation_payload! { CreditFacilityAgreementGeneratePayload, credit_facility_agreement: CreditFacilityAgreement }

#[derive(InputObject)]
pub struct CreditFacilityAgreementDownloadLinksGenerateInput {
    pub credit_facility_agreement_id: UUID,
}

#[derive(SimpleObject)]
pub struct CreditFacilityAgreementDownloadLinksGeneratePayload {
    pub link: String,
}

impl From<lana_app::document::GeneratedDocumentDownloadLink>
    for CreditFacilityAgreementDownloadLinksGeneratePayload
{
    fn from(value: lana_app::document::GeneratedDocumentDownloadLink) -> Self {
        Self { link: value.link }
    }
}
