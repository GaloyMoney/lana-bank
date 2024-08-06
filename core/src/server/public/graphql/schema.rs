use async_graphql::*;

use super::withdraw::*;
use crate::{
    app::LavaApp,
    primitives::LoanId,
    server::{
        public::PublicAuthContext,
        shared_graphql::{
            customer::Customer,
            loan::Loan,
            primitives::UUID,
            sumsub::{SumsubPermalinkCreatePayload, SumsubTokenCreatePayload},
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
    pub async fn withdrawal_initiate(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalInitiateInput,
    ) -> async_graphql::Result<WithdrawalInitiatePayload> {
        let PublicAuthContext { customer_id } = ctx.data()?;

        let app = ctx.data_unchecked::<LavaApp>();

        let withdraw = app
            .withdraws()
            .initiate(*customer_id, input.amount, input.reference)
            .await?;

        Ok(WithdrawalInitiatePayload::from(withdraw))
    }

    pub async fn sumsub_token_create(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<SumsubTokenCreatePayload> {
        let PublicAuthContext { customer_id } = ctx.data()?;

        let app = ctx.data_unchecked::<LavaApp>();
        let res = app.applicants().create_access_token(*customer_id).await?;

        Ok(SumsubTokenCreatePayload { token: res.token })
    }

    pub async fn sumsub_permalink_create(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<SumsubPermalinkCreatePayload> {
        let PublicAuthContext { customer_id } = ctx.data()?;

        let app = ctx.data_unchecked::<LavaApp>();
        let res = app.applicants().create_permalink(*customer_id).await?;

        let url = res.url;
        Ok(SumsubPermalinkCreatePayload { url })
    }
}
