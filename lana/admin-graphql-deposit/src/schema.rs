use admin_graphql_shared::primitives::*;
use async_graphql::{Context, Object, types::connection::*};

use super::*;

#[derive(Default)]
pub struct DepositQuery;

#[Object]
impl DepositQuery {
    async fn withdrawal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Withdrawal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Withdrawal,
            ctx,
            app.deposits().find_withdrawal_by_id(sub, id)
        )
    }

    async fn withdrawal_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<Withdrawal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Withdrawal,
            ctx,
            app.deposits().find_withdrawal_by_public_id(sub, id)
        )
    }

    async fn withdrawals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<WithdrawalsByCreatedAtCursor, Withdrawal, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            WithdrawalsByCreatedAtCursor,
            Withdrawal,
            ctx,
            after,
            first,
            |query| app.deposits().list_withdrawals(sub, query)
        )
    }

    async fn deposit(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Deposit>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Deposit, ctx, app.deposits().find_deposit_by_id(sub, id))
    }

    async fn deposit_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<Deposit>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Deposit,
            ctx,
            app.deposits().find_deposit_by_public_id(sub, id)
        )
    }

    async fn deposit_account(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<DepositAccount>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            DepositAccount,
            ctx,
            app.deposits().find_account_by_id(sub, id)
        )
    }

    async fn deposit_account_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<DepositAccount>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            DepositAccount,
            ctx,
            app.deposits().find_account_by_public_id(sub, id)
        )
    }

    async fn deposit_accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DepositAccountsByCreatedAtCursor, DepositAccount, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DepositAccountsByCreatedAtCursor,
            DepositAccount,
            ctx,
            after,
            first,
            |query| app.deposits().list_accounts(sub, query)
        )
    }

    async fn deposits(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DepositsByCreatedAtCursor, Deposit, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DepositsByCreatedAtCursor,
            Deposit,
            ctx,
            after,
            first,
            |query| app.deposits().list_deposits(sub, query)
        )
    }
}

#[derive(Default)]
pub struct DepositMutation;

#[Object]
impl DepositMutation {
    pub async fn deposit_record(
        &self,
        ctx: &Context<'_>,
        input: DepositRecordInput,
    ) -> async_graphql::Result<DepositRecordPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        exec_mutation!(
            DepositRecordPayload,
            Deposit,
            ctx,
            app.deposits().record_deposit(
                sub,
                input.deposit_account_id,
                input.amount,
                input.reference
            )
        )
    }

    pub async fn withdrawal_initiate(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalInitiateInput,
    ) -> async_graphql::Result<WithdrawalInitiatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            WithdrawalInitiatePayload,
            Withdrawal,
            ctx,
            app.deposits().initiate_withdrawal(
                sub,
                input.deposit_account_id,
                input.amount,
                input.reference
            )
        )
    }

    pub async fn withdrawal_confirm(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalConfirmInput,
    ) -> async_graphql::Result<WithdrawalConfirmPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        exec_mutation!(
            WithdrawalConfirmPayload,
            Withdrawal,
            ctx,
            app.deposits().confirm_withdrawal(sub, input.withdrawal_id)
        )
    }

    pub async fn withdrawal_cancel(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalCancelInput,
    ) -> async_graphql::Result<WithdrawalCancelPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            WithdrawalCancelPayload,
            Withdrawal,
            ctx,
            app.deposits().cancel_withdrawal(sub, input.withdrawal_id)
        )
    }

    pub async fn withdrawal_revert(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalRevertInput,
    ) -> async_graphql::Result<WithdrawalRevertPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            WithdrawalRevertPayload,
            Withdrawal,
            ctx,
            app.deposits().revert_withdrawal(sub, input.withdrawal_id)
        )
    }

    pub async fn deposit_revert(
        &self,
        ctx: &Context<'_>,
        input: DepositRevertInput,
    ) -> async_graphql::Result<DepositRevertPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DepositRevertPayload,
            Deposit,
            ctx,
            app.deposits().revert_deposit(sub, input.deposit_id)
        )
    }

    pub async fn deposit_account_create(
        &self,
        ctx: &Context<'_>,
        input: DepositAccountCreateInput,
    ) -> async_graphql::Result<DepositAccountCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        exec_mutation!(
            DepositAccountCreatePayload,
            DepositAccount,
            ctx,
            app.deposits().create_account(sub, input.customer_id)
        )
    }

    pub async fn deposit_account_freeze(
        &self,
        ctx: &Context<'_>,
        input: DepositAccountFreezeInput,
    ) -> async_graphql::Result<DepositAccountFreezePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DepositAccountFreezePayload,
            DepositAccount,
            ctx,
            app.deposits().freeze_account(sub, input.deposit_account_id)
        )
    }

    pub async fn deposit_account_unfreeze(
        &self,
        ctx: &Context<'_>,
        input: DepositAccountUnfreezeInput,
    ) -> async_graphql::Result<DepositAccountUnfreezePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DepositAccountUnfreezePayload,
            DepositAccount,
            ctx,
            app.deposits()
                .unfreeze_account(sub, input.deposit_account_id)
        )
    }

    pub async fn deposit_account_close(
        &self,
        ctx: &Context<'_>,
        input: DepositAccountCloseInput,
    ) -> async_graphql::Result<DepositAccountClosePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DepositAccountClosePayload,
            DepositAccount,
            ctx,
            app.deposits().close_account(sub, input.deposit_account_id)
        )
    }
}
