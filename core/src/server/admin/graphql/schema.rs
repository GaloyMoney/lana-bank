use async_graphql::{types::connection::*, *};

use super::user::*;
use crate::{
    app::LavaApp,
    primitives::{FixedTermLoanId, UserId, WithdrawId},
    server::shared_graphql::{fixed_term_loan::FixedTermLoan, primitives::UUID, user::User},
};

pub struct Query;

#[Object]
impl Query {
    async fn loan(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<FixedTermLoan>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let loan = app
            .fixed_term_loans()
            .find_by_id(FixedTermLoanId::from(id))
            .await?;
        Ok(loan.map(FixedTermLoan::from))
    }

    async fn user(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<User>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app.users().find_by_id(UserId::from(id)).await?;
        Ok(user.map(User::from))
    }

    async fn loans_for_user(
        &self,
        ctx: &Context<'_>,
        user_id: UUID,
    ) -> async_graphql::Result<Option<Vec<FixedTermLoan>>> {
        let app = ctx.data_unchecked::<LavaApp>();
        if let Some(loans) = app.list_loans_for_user(UserId::from(user_id)).await? {
            return Ok(Some(loans.into_iter().map(FixedTermLoan::from).collect()));
        }
        Ok(None)
    }

    async fn users(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> Result<Connection<UserByNameCursor, User, EmptyFields, EmptyFields>> {
        let app = ctx.data_unchecked::<LavaApp>();
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .users()
                    .list(crate::query::PaginatedQueryArgs {
                        first,
                        after: after.map(crate::user::UserByNameCursor::from),
                    })
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|user| {
                        let cursor =
                            UserByNameCursor::from((user.id, user.bitfinex_username.as_ref()));
                        Edge::new(cursor, User::from(user))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn user_pledge_collateral(
        &self,
        ctx: &Context<'_>,
        input: UserPledgeCollateralInput,
    ) -> async_graphql::Result<UserPledgeCollateralPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        println!("user_pledge_collateral");
        Ok(UserPledgeCollateralPayload::from(
            app.users()
                .pledge_unallocated_collateral_for_user(
                    UserId::from(input.user_id),
                    input.amount,
                    input.reference,
                )
                .await?,
        ))
    }

    pub async fn user_deposit(
        &self,
        ctx: &Context<'_>,
        input: UserDepositInput,
    ) -> async_graphql::Result<UserDepositPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        Ok(UserDepositPayload::from(
            app.users()
                .deposit_checking_for_user(
                    UserId::from(input.user_id),
                    input.amount,
                    input.reference,
                )
                .await?,
        ))
    }

    pub async fn withdrawal_settle(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalSettleInput,
    ) -> async_graphql::Result<WithdrawalSettlePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        Ok(WithdrawalSettlePayload::from(
            app.withdraws()
                .settle(WithdrawId::from(input.withdrawal_id), input.reference)
                .await?,
        ))
    }
}
