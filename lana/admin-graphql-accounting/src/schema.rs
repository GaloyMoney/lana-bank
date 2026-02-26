use async_graphql::{Context, Error, Object, Subscription, types::connection::*};
use futures::{StreamExt, stream::Stream};
use obix::out::OutboxEventMarker;

use super::*;
use lana_app::accounting::{CoreAccountingEvent, LedgerAccountId};
use std::io::Read;

const CHART_REF: &str = lana_app::accounting_init::constants::CHART_REF;
const BALANCE_SHEET_NAME: &str = lana_app::accounting_init::constants::BALANCE_SHEET_NAME;
const PROFIT_AND_LOSS_STATEMENT_NAME: &str =
    lana_app::accounting_init::constants::PROFIT_AND_LOSS_STATEMENT_NAME;
const TRIAL_BALANCE_STATEMENT_NAME: &str =
    lana_app::accounting_init::constants::TRIAL_BALANCE_STATEMENT_NAME;

#[derive(Default)]
pub struct AccountingQuery;

#[Object]
impl AccountingQuery {
    async fn transaction_templates(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<TransactionTemplateCursor, TransactionTemplate, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            TransactionTemplateCursor,
            TransactionTemplate,
            after,
            first,
            |query| app.accounting().transaction_templates().list(sub, query)
        )
    }

    async fn fiscal_year(
        &self,
        ctx: &Context<'_>,
        fiscal_year_id: admin_graphql_shared::primitives::UUID,
    ) -> async_graphql::Result<Option<FiscalYear>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            FiscalYear,
            app.accounting()
                .fiscal_year()
                .find_by_id(sub, fiscal_year_id)
        )
    }

    async fn fiscal_year_by_year(
        &self,
        ctx: &Context<'_>,
        year: String,
    ) -> async_graphql::Result<Option<FiscalYear>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            FiscalYear,
            app.accounting()
                .find_fiscal_year_for_chart_by_year(sub, CHART_REF, &year)
        )
    }

    async fn fiscal_years(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<FiscalYearsByCreatedAtCursor, FiscalYear, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        list_with_cursor!(
            FiscalYearsByCreatedAtCursor,
            FiscalYear,
            after,
            first,
            |query| app
                .accounting()
                .list_fiscal_years_for_chart(sub, CHART_REF, query,)
        )
    }

    async fn account_entry_csv(
        &self,
        ctx: &Context<'_>,
        ledger_account_id: admin_graphql_shared::primitives::UUID,
    ) -> async_graphql::Result<Option<AccountingCsvDocument>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let latest = app
            .accounting()
            .csvs()
            .get_latest_for_ledger_account_id(sub, ledger_account_id)
            .await?
            .map(AccountingCsvDocument::from);

        Ok(latest)
    }

    async fn chart_of_accounts(&self, ctx: &Context<'_>) -> async_graphql::Result<ChartOfAccounts> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let chart = app
            .accounting()
            .chart_of_accounts()
            .find_by_reference_with_sub(sub, CHART_REF)
            .await?;
        Ok(ChartOfAccounts::from(chart))
    }

    async fn descendant_account_sets_by_category(
        &self,
        ctx: &Context<'_>,
        category: AccountCategory,
    ) -> async_graphql::Result<Vec<AccountInfo>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let members = app
            .accounting()
            .chart_of_accounts()
            .descendant_account_sets_by_category(sub, CHART_REF, category.into())
            .await?;
        Ok(members.into_iter().map(AccountInfo::from).collect())
    }
}

#[derive(Default)]
pub struct AccountingMutation;

#[Object]
impl AccountingMutation {
    async fn fiscal_year_init(
        &self,
        ctx: &Context<'_>,
        input: FiscalYearInitInput,
    ) -> async_graphql::Result<FiscalYearInitPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            FiscalYearInitPayload,
            FiscalYear,
            app.accounting()
                .init_fiscal_year_for_chart(sub, CHART_REF, input.opened_as_of)
        )
    }

    async fn fiscal_year_close_month(
        &self,
        ctx: &Context<'_>,
        input: FiscalYearCloseMonthInput,
    ) -> async_graphql::Result<FiscalYearCloseMonthPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            FiscalYearCloseMonthPayload,
            FiscalYear,
            app.accounting()
                .fiscal_year()
                .close_month(sub, input.fiscal_year_id)
        )
    }

    async fn fiscal_year_open_next(
        &self,
        ctx: &Context<'_>,
        input: FiscalYearOpenNextInput,
    ) -> async_graphql::Result<FiscalYearOpenNextPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            FiscalYearOpenNextPayload,
            FiscalYear,
            app.accounting()
                .fiscal_year()
                .open_next(sub, input.fiscal_year_id)
        )
    }

    async fn fiscal_year_close(
        &self,
        ctx: &Context<'_>,
        input: FiscalYearCloseInput,
    ) -> async_graphql::Result<FiscalYearClosePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            FiscalYearClosePayload,
            FiscalYear,
            app.accounting()
                .fiscal_year()
                .close(sub, input.fiscal_year_id)
        )
    }

    async fn chart_of_accounts_csv_import(
        &self,
        ctx: &Context<'_>,
        input: ChartOfAccountsCsvImportInput,
    ) -> async_graphql::Result<ChartOfAccountsCsvImportPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let mut file = input.file.value(ctx)?.content;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        exec_mutation!(
            ChartOfAccountsCsvImportPayload,
            ChartOfAccounts,
            app.accounting()
                .import_csv(sub, CHART_REF, data, TRIAL_BALANCE_STATEMENT_NAME)
        )
    }

    async fn chart_of_accounts_add_root_node(
        &self,
        ctx: &Context<'_>,
        input: ChartOfAccountsAddRootNodeInput,
    ) -> async_graphql::Result<ChartOfAccountsAddRootNodePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ChartOfAccountsAddRootNodePayload,
            ChartOfAccounts,
            app.accounting().add_root_node(
                sub,
                CHART_REF,
                input.try_into()?,
                TRIAL_BALANCE_STATEMENT_NAME,
            )
        )
    }

    async fn chart_of_accounts_add_child_node(
        &self,
        ctx: &Context<'_>,
        input: ChartOfAccountsAddChildNodeInput,
    ) -> async_graphql::Result<ChartOfAccountsAddChildNodePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ChartOfAccountsAddChildNodePayload,
            ChartOfAccounts,
            app.accounting().add_child_node(
                sub,
                CHART_REF,
                input.parent.try_into()?,
                input.code.try_into()?,
                input.name.parse()?,
                TRIAL_BALANCE_STATEMENT_NAME,
            )
        )
    }

    async fn chart_of_accounts_csv_import_with_base_config(
        &self,
        ctx: &Context<'_>,
        input: ChartOfAccountsCsvImportWithBaseConfigInput,
    ) -> async_graphql::Result<ChartOfAccountsCsvImportWithBaseConfigPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let mut file = input.file.value(ctx)?.content;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        exec_mutation!(
            ChartOfAccountsCsvImportWithBaseConfigPayload,
            ChartOfAccounts,
            app.accounting().import_csv_with_base_config(
                sub,
                CHART_REF,
                data,
                input.base_config.try_into()?,
                BALANCE_SHEET_NAME,
                PROFIT_AND_LOSS_STATEMENT_NAME,
                TRIAL_BALANCE_STATEMENT_NAME
            )
        )
    }

    async fn ledger_account_csv_create(
        &self,
        ctx: &Context<'_>,
        input: LedgerAccountCsvCreateInput,
    ) -> async_graphql::Result<LedgerAccountCsvCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let csv = app
            .accounting()
            .csvs()
            .create_ledger_account_csv(sub, input.ledger_account_id)
            .await?;

        let csv_document = AccountingCsvDocument::from(csv);
        Ok(LedgerAccountCsvCreatePayload::from(csv_document))
    }

    async fn accounting_csv_download_link_generate(
        &self,
        ctx: &Context<'_>,
        input: AccountingCsvDownloadLinkGenerateInput,
    ) -> async_graphql::Result<AccountingCsvDownloadLinkGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let result = app
            .accounting()
            .csvs()
            .generate_download_link(sub, input.document_id.into())
            .await?;

        let link = AccountingCsvDownloadLink::from(result);

        Ok(AccountingCsvDownloadLinkGeneratePayload::from(link))
    }
}

#[derive(Default)]
pub struct AccountingSubscription;

#[Subscription]
impl AccountingSubscription {
    async fn ledger_account_csv_export_uploaded(
        &self,
        ctx: &Context<'_>,
        ledger_account_id: admin_graphql_shared::primitives::UUID,
    ) -> async_graphql::Result<impl Stream<Item = LedgerAccountCsvExportUploadedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let ledger_account_id = LedgerAccountId::from(ledger_account_id);

        app.accounting()
            .find_ledger_account_by_id(sub, CHART_REF, ledger_account_id)
            .await?
            .ok_or_else(|| Error::new("Ledger account not found"))?;

        let stream = app.outbox().listen_ephemeral();
        let updates = stream.filter_map(move |event| async move {
            let event: &CoreAccountingEvent = event.payload.as_event()?;
            match event {
                CoreAccountingEvent::LedgerAccountCsvExportUploaded {
                    id,
                    ledger_account_id: event_ledger_account_id,
                } if *event_ledger_account_id == ledger_account_id => {
                    Some(LedgerAccountCsvExportUploadedPayload {
                        document_id: admin_graphql_shared::primitives::UUID::from(*id),
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }
}
