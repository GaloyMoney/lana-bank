use async_graphql::*;

use crate::server::shared_graphql::primitives::{Timestamp, UsdCents, UUID};

use super::convert::ToGlobalId;

#[derive(InputObject)]
pub struct LoanCreateInput {
    pub user_id: UUID,
    pub desired_principal: UsdCents,
}

#[derive(SimpleObject)]
pub struct Loan {
    id: ID,
    loan_id: UUID,
    start_date: Timestamp,
    #[graphql(skip)]
    _user_id: UUID,
    #[graphql(skip)]
    _account_ids: crate::ledger::loan::LoanAccountIds,
}

impl ToGlobalId for crate::primitives::LoanId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("loan:{}", self))
    }
}

impl From<crate::loan::Loan> for Loan {
    fn from(loan: crate::loan::Loan) -> Self {
        Loan {
            id: loan.id.to_global_id(),
            loan_id: UUID::from(loan.id),
            _user_id: UUID::from(loan.user_id),
            _account_ids: loan.account_ids,
            start_date: Timestamp::from(loan.start_date),
        }
    }
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
pub struct LoanPartialPaymentInput {
    pub loan_id: UUID,
    pub amount: UsdCents,
}

#[derive(SimpleObject)]
pub struct LoanPartialPaymentPayload {
    loan: Loan,
}

impl From<crate::loan::Loan> for LoanPartialPaymentPayload {
    fn from(loan: crate::loan::Loan) -> Self {
        Self { loan: loan.into() }
    }
}
