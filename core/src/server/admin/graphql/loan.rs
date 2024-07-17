use async_graphql::*;

use crate::server::shared_graphql::{
    loan::*,
    primitives::{Satoshis, UsdCents, UUID},
};

#[derive(InputObject)]
pub struct LoanCreateInput {
    pub user_id: UUID,
    pub desired_principal: UsdCents,
}

#[derive(SimpleObject)]
pub struct LoanCreatePayload {
    loan: Loan,
}

impl From<crate::loan::Loan> for LoanCreatePayload {
    fn from(loan: crate::loan::Loan) -> Self {
        Self { loan: loan.into() }
    }
}

#[derive(InputObject)]
pub struct LoanApproveInput {
    pub loan_id: UUID,
    pub collateral: Satoshis,
}

#[derive(SimpleObject)]
pub struct LoanApprovePayload {
    loan: Loan,
}

impl From<crate::loan::Loan> for LoanApprovePayload {
    fn from(loan: crate::loan::Loan) -> Self {
        Self { loan: loan.into() }
    }
}
