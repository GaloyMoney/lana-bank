use std::io::Read;

use async_graphql::{Context, Object, types::connection::*};

use admin_graphql_shared::primitives::*;

use lana_app::accounting_init::constants::{
    BALANCE_SHEET_NAME, PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME,
};

use crate::{
    balance_sheet::*, chart_of_accounts::*, csv::*, fiscal_year::*, journal_entry::*,
    ledger_account::*, ledger_transaction::*, manual_transaction::*, profit_and_loss::*,
    transaction_templates::*, trial_balance::*,
};

#[derive(Default)]
pub struct AccountingQuery;

#[Object]
impl AccountingQuery {
    async fn ledger_account(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<LedgerAccount>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            LedgerAccount,
            app.accounting()
                .find_ledger_account_by_id(sub, CHART_REF, id)
        )
    }

    async fn ledger_account_by_code(
        &self,
        ctx: &Context<'_>,
        code: String,
    ) -> async_graphql::Result<Option<LedgerAccount>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            LedgerAccount,
            app.accounting()
                .find_ledger_account_by_code(sub, CHART_REF, code)
        )
    }

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

    async fn ledger_transaction(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<LedgerTransaction>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            LedgerTransaction,
            app.accounting().ledger_transactions().find_by_id(sub, id)
        )
    }

    async fn ledger_transactions_for_template_code(
        &self,
        ctx: &Context<'_>,
        template_code: String,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<LedgerTransactionCursor, LedgerTransaction, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            LedgerTransactionCursor,
            LedgerTransaction,
            after,
            first,
            |query| app
                .accounting()
                .ledger_transactions()
                .list_for_template_code(sub, &template_code, query)
        )
    }

    async fn journal_entries(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<JournalEntryCursor, JournalEntry, EmptyFields, EmptyFields>>
    {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let query_args = es_entity::PaginatedQueryArgs { first, after };
                let res = app.accounting().journal().entries(sub, query_args).await?;

                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entry| {
                        let cursor = JournalEntryCursor::from(&entry);
                        Edge::new(cursor, JournalEntry::from(entry))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn trial_balance(
        &self,
        ctx: &Context<'_>,
        from: Date,
        until: Date,
    ) -> async_graphql::Result<TrialBalance> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let account_summary = app
            .accounting()
            .trial_balances()
            .trial_balance(
                sub,
                TRIAL_BALANCE_STATEMENT_NAME.to_string(),
                from.into_inner(),
                until.into_inner(),
            )
            .await?;
        Ok(TrialBalance::from(account_summary))
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

    async fn fiscal_year(
        &self,
        ctx: &Context<'_>,
        fiscal_year_id: UUID,
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
                .list_fiscal_years_for_chart(sub, CHART_REF, query)
        )
    }

    async fn balance_sheet(
        &self,
        ctx: &Context<'_>,
        from: Date,
        until: Option<Date>,
    ) -> async_graphql::Result<BalanceSheet> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let balance_sheet = app
            .accounting()
            .balance_sheets()
            .balance_sheet(
                sub,
                BALANCE_SHEET_NAME.to_string(),
                from.into_inner(),
                until.map(|t| t.into_inner()),
            )
            .await?;
        Ok(BalanceSheet::from(balance_sheet))
    }

    async fn profit_and_loss_statement(
        &self,
        ctx: &Context<'_>,
        from: Date,
        until: Option<Date>,
    ) -> async_graphql::Result<ProfitAndLossStatement> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let profit_and_loss = app
            .accounting()
            .profit_and_loss()
            .pl_statement(
                sub,
                PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
                from.into_inner(),
                until.map(|t| t.into_inner()),
            )
            .await?;
        Ok(ProfitAndLossStatement::from(profit_and_loss))
    }

    async fn account_entry_csv(
        &self,
        ctx: &Context<'_>,
        ledger_account_id: UUID,
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
}

#[derive(Default)]
pub struct AccountingMutation;

#[Object]
impl AccountingMutation {
    pub async fn manual_transaction_execute(
        &self,
        ctx: &Context<'_>,
        input: ManualTransactionExecuteInput,
    ) -> async_graphql::Result<ManualTransactionExecutePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let mut entries = Vec::with_capacity(input.entries.len());
        for entry in input.entries.into_iter() {
            entries.push(entry.try_into()?);
        }

        exec_mutation!(
            ManualTransactionExecutePayload,
            LedgerTransaction,
            app.accounting().execute_manual_transaction(
                sub,
                CHART_REF,
                input.reference,
                input.description,
                input.effective.map(|ts| ts.into_inner()),
                entries
            )
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

    pub async fn ledger_account_csv_create(
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

    pub async fn accounting_csv_download_link_generate(
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
