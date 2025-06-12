use async_graphql::*;

use crate::primitives::*;

#[derive(async_graphql::Enum, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LoanAgreementStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(SimpleObject, Clone)]
pub struct LoanAgreement {
    id: ID,
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
            id: id.to_string().into(),
            status,
            created_at: created_at.into(),
        }
    }
}

impl From<lana_app::contract_creation::SimpleLoanAgreement> for LoanAgreement {
    fn from(simple_loan_agreement: lana_app::contract_creation::SimpleLoanAgreement) -> Self {
        let status = match simple_loan_agreement.status {
            lana_app::contract_creation::SimpleLoanAgreementStatus::Pending => {
                LoanAgreementStatus::Pending
            }
            lana_app::contract_creation::SimpleLoanAgreementStatus::Completed => {
                LoanAgreementStatus::Completed
            }
            lana_app::contract_creation::SimpleLoanAgreementStatus::Failed => {
                LoanAgreementStatus::Failed
            }
        };

        Self::new(
            simple_loan_agreement.id,
            status,
            simple_loan_agreement.created_at,
        )
    }
}

#[derive(InputObject)]
pub struct LoanAgreementGenerateInput {
    pub customer_id: UUID,
}

crate::mutation_payload! { LoanAgreementGeneratePayload, loan_agreement: LoanAgreement }
