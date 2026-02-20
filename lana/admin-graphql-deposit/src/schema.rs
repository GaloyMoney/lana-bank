use async_graphql::{Context, Object, types::connection::*};

use admin_graphql_shared::primitives::UUID;

use crate::{
    deposit::*, deposit_account::*, deposit_config::*, ledger_accounts::CHART_REF, withdrawal::*,
};

#[derive(Default)]
pub struct DepositQuery;

#[Object]
impl DepositQuery {
    async fn withdrawal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<WithdrawalBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            WithdrawalBase,
            app.deposits().find_withdrawal_by_id(sub, id)
        )
    }

    async fn withdrawal_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<WithdrawalBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            WithdrawalBase,
            app.deposits().find_withdrawal_by_public_id(sub, id)
        )
    }

    async fn withdrawals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<WithdrawalsByCreatedAtCursor, WithdrawalBase, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            WithdrawalsByCreatedAtCursor,
            WithdrawalBase,
            after,
            first,
            |query| app.deposits().list_withdrawals(sub, query)
        )
    }

    async fn deposit(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<DepositBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(DepositBase, app.deposits().find_deposit_by_id(sub, id))
    }

    async fn deposit_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<DepositBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            DepositBase,
            app.deposits().find_deposit_by_public_id(sub, id)
        )
    }

    async fn deposit_account(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<DepositAccountBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            DepositAccountBase,
            app.deposits().find_account_by_id(sub, id)
        )
    }

    async fn deposit_account_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<DepositAccountBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            DepositAccountBase,
            app.deposits().find_account_by_public_id(sub, id)
        )
    }

    async fn deposit_accounts(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DepositAccountsByCreatedAtCursor, DepositAccountBase, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DepositAccountsByCreatedAtCursor,
            DepositAccountBase,
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
        Connection<DepositsByCreatedAtCursor, DepositBase, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DepositsByCreatedAtCursor,
            DepositBase,
            after,
            first,
            |query| app.deposits().list_deposits(sub, query)
        )
    }

    async fn deposit_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<DepositModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .deposits()
            .chart_of_accounts_integrations()
            .get_config(sub)
            .await?;
        Ok(config.map(DepositModuleConfig::from))
    }
}

mutation_payload! { DepositRecordPayload, deposit: DepositBase }
mutation_payload! { DepositRevertPayload, deposit: DepositBase }
mutation_payload! { DepositAccountCreatePayload, account: DepositAccountBase }
mutation_payload! { DepositAccountFreezePayload, account: DepositAccountBase }
mutation_payload! { DepositAccountUnfreezePayload, account: DepositAccountBase }
mutation_payload! { DepositAccountClosePayload, account: DepositAccountBase }
mutation_payload! { WithdrawalInitiatePayload, withdrawal: WithdrawalBase }
mutation_payload! { WithdrawalConfirmPayload, withdrawal: WithdrawalBase }
mutation_payload! { WithdrawalCancelPayload, withdrawal: WithdrawalBase }
mutation_payload! { WithdrawalRevertPayload, withdrawal: WithdrawalBase }
mutation_payload! { DepositModuleConfigurePayload, deposit_config: DepositModuleConfig }

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
            DepositBase,
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
            WithdrawalBase,
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
            WithdrawalBase,
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
            WithdrawalBase,
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
            WithdrawalBase,
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
            DepositBase,
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
            DepositAccountBase,
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
            DepositAccountBase,
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
            DepositAccountBase,
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
            DepositAccountBase,
            app.deposits().close_account(sub, input.deposit_account_id)
        )
    }

    async fn deposit_module_configure(
        &self,
        ctx: &Context<'_>,
        input: DepositModuleConfigureInput,
    ) -> async_graphql::Result<DepositModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let chart = app
            .accounting()
            .chart_of_accounts()
            .maybe_find_by_reference(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {CHART_REF:?}"));

        let DepositModuleConfigureInput {
            chart_of_accounts_omnibus_parent_code,
            chart_of_accounts_individual_deposit_accounts_parent_code,
            chart_of_accounts_government_entity_deposit_accounts_parent_code,
            chart_of_account_private_company_deposit_accounts_parent_code,
            chart_of_account_bank_deposit_accounts_parent_code,
            chart_of_account_financial_institution_deposit_accounts_parent_code,
            chart_of_account_non_domiciled_company_deposit_accounts_parent_code,
            chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            chart_of_account_frozen_bank_deposit_accounts_parent_code,
            chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code,
        } = input;

        let config_values = lana_app::deposit::ChartOfAccountsIntegrationConfig {
            chart_of_accounts_id: chart.id,
            chart_of_accounts_individual_deposit_accounts_parent_code:
                chart_of_accounts_individual_deposit_accounts_parent_code.parse()?,
            chart_of_accounts_government_entity_deposit_accounts_parent_code:
                chart_of_accounts_government_entity_deposit_accounts_parent_code.parse()?,
            chart_of_account_private_company_deposit_accounts_parent_code:
                chart_of_account_private_company_deposit_accounts_parent_code.parse()?,
            chart_of_account_bank_deposit_accounts_parent_code:
                chart_of_account_bank_deposit_accounts_parent_code.parse()?,
            chart_of_account_financial_institution_deposit_accounts_parent_code:
                chart_of_account_financial_institution_deposit_accounts_parent_code.parse()?,
            chart_of_account_non_domiciled_company_deposit_accounts_parent_code:
                chart_of_account_non_domiciled_company_deposit_accounts_parent_code.parse()?,
            chart_of_accounts_frozen_individual_deposit_accounts_parent_code:
                chart_of_accounts_frozen_individual_deposit_accounts_parent_code.parse()?,
            chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code:
                chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code.parse()?,
            chart_of_account_frozen_private_company_deposit_accounts_parent_code:
                chart_of_account_frozen_private_company_deposit_accounts_parent_code.parse()?,
            chart_of_account_frozen_bank_deposit_accounts_parent_code:
                chart_of_account_frozen_bank_deposit_accounts_parent_code.parse()?,
            chart_of_account_frozen_financial_institution_deposit_accounts_parent_code:
                chart_of_account_frozen_financial_institution_deposit_accounts_parent_code
                    .parse()?,
            chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code:
                chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code
                    .parse()?,
            chart_of_accounts_omnibus_parent_code: chart_of_accounts_omnibus_parent_code.parse()?,
        };

        let config = app
            .deposits()
            .chart_of_accounts_integrations()
            .set_config(sub, &chart, config_values)
            .await?;
        Ok(DepositModuleConfigurePayload::from(
            DepositModuleConfig::from(config),
        ))
    }
}
