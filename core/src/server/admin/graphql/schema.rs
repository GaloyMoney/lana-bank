use async_graphql::*;

use super::{fixed_term_loan::*, user::*};
use crate::{
    app::LavaApp,
    primitives::{FixedTermLoanId, UserId, WithdrawId},
    server::shared::primitives::UUID,
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
