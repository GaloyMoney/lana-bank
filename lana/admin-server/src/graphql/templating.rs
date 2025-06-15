use async_graphql::*;

use crate::primitives::*;

#[derive(InputObject)]
pub struct LoanAgreementGenerateInput {
    pub customer_id: UUID,
}

#[derive(SimpleObject)]
pub struct LoanAgreementGeneratePayload {
    pub customer_id: UUID,
    pub pdf_bytes: Vec<u8>,
    pub filename: String,
}

impl LoanAgreementGeneratePayload {
    pub fn new(customer_id: UUID, pdf_bytes: Vec<u8>) -> Self {
        let filename = format!("loan_agreement_{:?}.pdf", customer_id);
        Self {
            customer_id,
            pdf_bytes,
            filename,
        }
    }
}
