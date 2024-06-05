use async_graphql::*;

use crate::{
    app::LavaApp, ledger::fixed_term_loan::FixedTermLoanAccountIds, server::shared::primitives::*,
};

use super::fixed_term_loan_balance::FixedTermLoanBalance;

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
