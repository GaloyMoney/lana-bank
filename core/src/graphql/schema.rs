use async_graphql::*;

use super::{fixed_term_loan::*, primitives::UUID};
use crate::{app::LavaApp, primitives::FixedTermLoanId};

pub struct Query;

#[Object]
impl Query {
    async fn loan(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<FixedTermLoan> {
        let app = ctx.data_unchecked::<LavaApp>();
        let loan = app
            .fixed_term_loans()
            .find_by_id(FixedTermLoanId::from(id))
            .await?;
        Ok(FixedTermLoan::from(loan))
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn fixed_term_loan_create(
        &self,
        ctx: &Context<'_>,
        _input: FixedTermLoanCreateInput,
    ) -> async_graphql::Result<FixedTermLoanCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let loan = app.fixed_term_loans().create_loan().await?;
        Ok(FixedTermLoanCreatePayload::from(loan))
    }
}
