use async_graphql::*;

use super::{money::*, primitives::*};
use crate::{app::LavaApp, fixed_term_loan::FixedTermLoanState, primitives::FixedTermLoanId};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct FixedTermLoan {
    pub loan_id: UUID,
    pub state: FixedTermLoanState,
}

#[ComplexObject]
impl FixedTermLoan {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<Money> {
        let app = ctx.data_unchecked::<LavaApp>();
        let money = app
            .fixed_term_loans()
            .balance_for_loan(FixedTermLoanId::from(&self.loan_id))
            .await?;
        Ok(Money::from(money))
    }
}

#[derive(InputObject)]
pub struct FixedTermLoanCreateInput {
    bitfinex_user_name: String,
}

#[derive(SimpleObject)]
pub struct FixedTermLoanCreatePayload {
    pub loan: FixedTermLoan,
}

#[derive(InputObject)]
pub struct FixedTermLoanDeclareCollateralizedInput {
    pub loan_id: UUID,
}

#[derive(SimpleObject)]
pub struct FixedTermLoanDeclareCollateralizedPayload {
    pub loan: FixedTermLoan,
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoan {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        FixedTermLoan {
            loan_id: UUID::from(loan.id),
            state: loan.state,
        }
    }
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoanCreatePayload {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        FixedTermLoanCreatePayload {
            loan: FixedTermLoan::from(loan),
        }
    }
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoanDeclareCollateralizedPayload {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        FixedTermLoanDeclareCollateralizedPayload {
            loan: FixedTermLoan::from(loan),
        }
    }
}
