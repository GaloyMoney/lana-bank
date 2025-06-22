use async_graphql::*;

use crate::primitives::*;
use lana_app::customer::CustomerId;

pub use lana_app::document_storage::loan_agreement::{LoanAgreement as DomainLoanAgreement, LoanAgreementStatus};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct LoanAgreement {
    id: ID,
    loan_agreement_id: UUID,
    customer_id: UUID,
    status: LoanAgreementStatus,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainLoanAgreement>,
}

impl LoanAgreement {
    pub fn loan_agreement_id(&self) -> lana_app::document_storage::LoanAgreementId {
        lana_app::document_storage::LoanAgreementId::from(self.entity.id)
    }
}

impl From<DomainLoanAgreement> for LoanAgreement {
    fn from(agreement: DomainLoanAgreement) -> Self {
        Self {
            id: agreement.id.to_global_id(),
            loan_agreement_id: UUID::from(agreement.id),
            customer_id: UUID::from(agreement.customer_id),
            status: agreement.status,
            entity: Arc::new(agreement),
        }
    }
}

#[ComplexObject]
impl LoanAgreement {
    async fn filename(&self) -> Option<&str> {
        self.entity.filename.as_deref()
    }
    
    async fn storage_path(&self) -> Option<&str> {
        self.entity.storage_path.as_deref()
    }
    
    async fn error_message(&self) -> Option<&str> {
        self.entity.error_message.as_deref()
    }
}

#[derive(InputObject)]
pub struct LoanAgreementGenerateInput {
    pub customer_id: UUID,
}

#[derive(SimpleObject)]
pub struct LoanAgreementGeneratePayload {
    customer_id: UUID,
    storage_path: Option<String>,
    filename: Option<String>,
}

impl From<DomainLoanAgreement> for LoanAgreementGeneratePayload {
    fn from(agreement: DomainLoanAgreement) -> Self {
        Self {
            customer_id: UUID::from(agreement.customer_id),
            storage_path: agreement.storage_path,
            filename: agreement.filename,
        }
    }
}

#[derive(InputObject)]
pub struct LoanAgreementDownloadLinkGenerateInput {
    pub loan_agreement_id: UUID,
}

#[derive(SimpleObject)]
pub struct LoanAgreementDownloadLinkGeneratePayload {
    loan_agreement_id: UUID,
    link: String,
}

impl From<lana_app::document_storage::GeneratedLoanAgreementDownloadLink>
    for LoanAgreementDownloadLinkGeneratePayload
{
    fn from(value: lana_app::document_storage::GeneratedLoanAgreementDownloadLink) -> Self {
        Self {
            loan_agreement_id: UUID::from(value.loan_agreement_id),
            link: value.link,
        }
    }
}