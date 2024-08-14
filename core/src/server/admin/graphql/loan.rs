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
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CursorType for LoanCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(&self).expect("could not serialize token");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

impl From<(LoanId, chrono::DateTime<chrono::Utc>)> for LoanCursor {
    fn from((id, created_at): (LoanId, chrono::DateTime<chrono::Utc>)) -> Self {
        Self { id, created_at }
    }
}

impl From<LoanCursor> for crate::loan::LoanCursor {
    fn from(cursor: LoanCursor) -> Self {
        Self {
            id: cursor.id,
            created_at: cursor.created_at,
        }
    }
}
