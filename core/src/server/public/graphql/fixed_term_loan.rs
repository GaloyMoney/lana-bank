use async_graphql::*;

use crate::server::shared_graphql::{fixed_term_loan::FixedTermLoan, primitives::*};

#[derive(SimpleObject)]
pub struct FixedTermLoanCreatePayload {
    loan: FixedTermLoan,
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoanCreatePayload {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        Self {
            loan: FixedTermLoan::from(loan),
        }
    }
}

#[derive(InputObject)]
pub struct FixedTermLoanApproveInput {
    pub loan_id: UUID,
    pub collateral: SignedSatoshis,
    pub principal: SignedUsdCents,
}

#[derive(SimpleObject)]
pub struct FixedTermLoanApprovePayload {
    loan: FixedTermLoan,
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoanApprovePayload {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        Self {
            loan: FixedTermLoan::from(loan),
        }
    }
}

#[derive(InputObject)]
pub struct FixedTermLoanRecordPaymentInput {
    pub loan_id: UUID,
    pub amount: SignedUsdCents,
}

#[derive(SimpleObject)]
pub struct FixedTermLoanRecordPaymentPayload {
    loan: FixedTermLoan,
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoanRecordPaymentPayload {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        Self {
            loan: FixedTermLoan::from(loan),
        }
    }
}
