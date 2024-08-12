use async_graphql::*;

use async_graphql::connection::CursorType;
use serde::{Deserialize, Serialize};

use crate::{
    primitives::LoanId,
    server::shared_graphql::{
        loan::*,
        primitives::{Satoshis, UsdCents, UUID},
        terms::*,
    },
};

#[derive(InputObject)]
pub struct LoanCreateInput {
    pub customer_id: UUID,
    pub desired_principal: UsdCents,
    pub loan_terms: TermsInput,
}

#[derive(InputObject)]
pub struct TermsInput {
    pub annual_rate: AnnualRatePct,
    pub interval: InterestInterval,
    pub liquidation_cvl: CVLPct,
    pub duration: DurationInput,
    pub margin_call_cvl: CVLPct,
    pub initial_cvl: CVLPct,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanCursor {
    pub id: LoanId,
}

impl CursorType for LoanCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        self.id.to_string()
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let id = s.parse::<LoanId>().map_err(|e| e.to_string())?;
        Ok(LoanCursor { id })
    }
}

impl From<LoanCursor> for crate::loan::LoanCursor {
    fn from(cursor: LoanCursor) -> Self {
        Self { id: cursor.id }
    }
}

impl From<LoanId> for LoanCursor {
    fn from(id: LoanId) -> Self {
        Self { id }
    }
}
