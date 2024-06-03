use async_graphql::*;

use crate::{app::LavaApp, ledger::fixed_term_loan::FixedTermLoanAccountIds};

use super::{fixed_term_loan_balance::*, primitives::*};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct FixedTermLoan {
    loan_id: UUID,
    user_id: UUID,
    #[graphql(skip)]
    account_ids: FixedTermLoanAccountIds,
}

#[ComplexObject]
impl FixedTermLoan {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<FixedTermLoanBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app
            .ledger()
            .get_fixed_term_loan_balance(self.account_ids)
            .await?;
        Ok(FixedTermLoanBalance::from(balance))
    }
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoan {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        FixedTermLoan {
            loan_id: UUID::from(loan.id),
            user_id: UUID::from(loan.user_id),
            account_ids: loan.account_ids,
        }
    }
}

#[derive(InputObject)]
pub struct FixedTermLoanCreateInput {
    pub user_id: UUID,
}

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
    pub collateral: Satoshis,
    pub principal: UsdCents,
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
    pub amount: UsdCents,
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

#[derive(InputObject)]
pub struct FixedTermLoanCompleteInput {
    pub loan_id: UUID,
}

#[derive(SimpleObject)]
pub struct FixedTermLoanCompletePayload {
    loan: FixedTermLoan,
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoanCompletePayload {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        Self {
            loan: FixedTermLoan::from(loan),
        }
    }
}
