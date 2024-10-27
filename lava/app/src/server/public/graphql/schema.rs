use async_graphql::*;

use crate::{
    app::LavaApp,
    primitives::LoanId,
    server::{
        public::PublicAuthContext,
        shared_graphql::{
            customer::Customer, loan::Loan, primitives::UUID, sumsub::SumsubTokenCreatePayload,
        },
    },
};

pub struct Query;

#[Object]
impl Query {
    async fn loan(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Loan>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let loan = app.loans().find_by_id(None, LoanId::from(id)).await?;
        Ok(loan.map(Loan::from))
    }

    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Customer>> {
        let PublicAuthContext { customer_id } = ctx.data()?;

        let app = ctx.data_unchecked::<LavaApp>();
        let user = app.customers().find_by_id(None, *customer_id).await?;

        Ok(user.map(Customer::from))
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn sumsub_token_create(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<SumsubTokenCreatePayload> {
        let PublicAuthContext { customer_id } = ctx.data()?;

        let app = ctx.data_unchecked::<LavaApp>();
        let res = app.applicants().create_access_token(*customer_id).await?;

        Ok(SumsubTokenCreatePayload { token: res.token })
    }
}
