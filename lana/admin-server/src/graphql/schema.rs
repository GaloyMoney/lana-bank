use async_graphql::{types::connection::*, Context, Object};

use std::io::Read;

use lana_app::{
    accounting::csv::AccountingCsvsByCreatedAtCursor,
    accounting_init::constants::{
        BALANCE_SHEET_NAME, PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME,
    },
    app::LanaApp,
};

use crate::primitives::*;

use super::{
    access::*, accounting::*, approval_process::*, audit::*, authenticated_subject::*,
    balance_sheet_config::*, committee::*, credit_config::*, credit_facility::*, custody::*,
    customer::*, dashboard::*, deposit::*, deposit_config::*, document::*, loader::*, policy::*,
    price::*, profit_and_loss_config::*, report::*, sumsub::*, terms_template::*, withdrawal::*,
};

pub struct Query;

#[Object]
impl Query {
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<AuthenticatedSubject> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let user = Arc::new(app.access().users().find_for_subject(sub).await?);
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        loader.feed_one(user.id, User::from(user.clone())).await;
        Ok(AuthenticatedSubject::from(user))
    }

    async fn dashboard(&self, ctx: &Context<'_>) -> async_graphql::Result<Dashboard> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let dashboard = app.dashboard().load(sub).await?;
        Ok(Dashboard::from(dashboard))
    }

    async fn user(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<User>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(User, ctx, app.access().users().find_by_id(sub, id))
    }

    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let users: Vec<_> = app
            .access()
            .users()
            .list_users(sub)
            .await?
            .into_iter()
            .map(User::from)
            .collect();
        loader
            .feed_many(users.iter().map(|u| (u.entity.id, u.clone())))
            .await;
        Ok(users)
    }

    async fn role(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Role>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Role, ctx, app.access().find_role_by_id(sub, id))
    }

    async fn roles(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<RolesByNameCursor, Role, EmptyFields, EmptyFields>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(RolesByNameCursor, Role, ctx, after, first, |query| app
            .access()
            .list_roles(sub, query))
    }

    async fn permission_sets(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<PermissionSetsByIdCursor, PermissionSet, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            PermissionSetsByIdCursor,
            PermissionSet,
            ctx,
            after,
            first,
            |query| app.access().list_permission_sets(sub, query)
        )
    }

    async fn customer(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Customer>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Customer, ctx, app.customers().find_by_id(sub, id))
    }

    async fn customer_by_email(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> async_graphql::Result<Option<Customer>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Customer, ctx, app.customers().find_by_email(sub, email))
    }

    async fn customers(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        #[graphql(default_with = "Some(CustomersSort::default())")] sort: Option<CustomersSort>,
        filter: Option<CustomersFilter>,
    ) -> async_graphql::Result<Connection<CustomersCursor, Customer, EmptyFields, EmptyFields>>
    {
        let (filter_field, status) = match filter {
            Some(filter) => (Some(filter.field), filter.status),
            None => (None, None),
        };
        let filter = match filter_field {
            None => FindManyCustomers::NoFilter,
            Some(CustomersFilterBy::AccountStatus) => {
                let status = status.ok_or(CustomerError::MissingValueForFilterField(
                    "status".to_string(),
                ))?;
                FindManyCustomers::WithStatus(status)
            }
        };

        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let sort = Sort {
            by: DomainCustomersSortBy::from(sort.unwrap_or_default()),
            direction: ListDirection::Descending,
        };
        list_with_combo_cursor!(
            CustomersCursor,
            Customer,
            sort.by,
            ctx,
            after,
            first,
            |query| app.customers().list(sub, query, filter, sort)
        )
    }

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

    async fn terms_template(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<TermsTemplate>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            TermsTemplate,
            ctx,
            app.credit().terms_templates().find_by_id(sub, id)
        )
    }

    async fn terms_templates(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<TermsTemplate>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let terms_templates = app.credit().terms_templates().list(sub).await?;
        Ok(terms_templates
            .into_iter()
            .map(TermsTemplate::from)
            .collect())
    }

    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacility>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacility,
            ctx,
            app.credit().facilities().find_by_id(sub, id)
        )
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        #[graphql(default_with = "Some(CreditFacilitiesSort::default())")] sort: Option<
            CreditFacilitiesSort,
        >,
        filter: Option<CreditFacilitiesFilter>,
    ) -> async_graphql::Result<
        Connection<CreditFacilitiesCursor, CreditFacility, EmptyFields, EmptyFields>,
    > {
        let (filter_field, status, collateralization_state) = match filter {
            Some(filter) => (
                Some(filter.field),
                filter.status,
                filter.collateralization_state,
            ),
            None => (None, None, None),
        };
        let filter = match filter_field {
            None => FindManyCreditFacilities::NoFilter,
            Some(CreditFacilitiesFilterBy::Status) => {
                let status = status.ok_or(CreditFacilityError::MissingValueForFilterField(
                    "status".to_string(),
                ))?;
                FindManyCreditFacilities::WithStatus(status)
            }
            Some(CreditFacilitiesFilterBy::CollateralizationState) => {
                let collateralization_state = collateralization_state.ok_or(
                    CreditFacilityError::MissingValueForFilterField(
                        "collateralization_state".to_string(),
                    ),
                )?;
                FindManyCreditFacilities::WithCollateralizationState(collateralization_state)
            }
        };

        let sort = sort.unwrap_or_default();
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_combo_cursor!(
            CreditFacilitiesCursor,
            CreditFacility,
            DomainCreditFacilitiesSortBy::from(sort),
            ctx,
            after,
            first,
            |query| app.credit().facilities().list(sub, query, filter, sort)
        )
    }

    async fn disbursal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacilityDisbursal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityDisbursal,
            ctx,
            app.credit().disbursals().find_by_id(sub, id)
        )
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DisbursalsCursor, CreditFacilityDisbursal, EmptyFields, EmptyFields>,
    > {
        let filter = FindManyDisbursals::NoFilter;

        let sort = Sort {
            by: DomainDisbursalsSortBy::CreatedAt,
            direction: ListDirection::Descending,
        };
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_combo_cursor!(
            DisbursalsCursor,
            CreditFacilityDisbursal,
            sort.by,
            ctx,
            after,
            first,
            |query| { app.credit().disbursals().list(sub, query, filter, sort) }
        )
    }

    async fn custodians(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<CustodiansByNameCursor, Custodian, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            CustodiansByNameCursor,
            Custodian,
            ctx,
            after,
            first,
            |query| app.custody().list_custodians(sub, query)
        )
    }

    async fn committee(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Committee>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Committee,
            ctx,
            app.governance().find_committee_by_id(sub, id)
        )
    }

    async fn committees(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<CommitteesByCreatedAtCursor, Committee, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            CommitteesByCreatedAtCursor,
            Committee,
            ctx,
            after,
            first,
            |query| app.governance().list_committees(sub, query)
        )
    }

    async fn policy(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Policy>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Policy, ctx, app.governance().find_policy(sub, id))
    }

    async fn policies(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<PoliciesByCreatedAtCursor, Policy, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            PoliciesByCreatedAtCursor,
            Policy,
            ctx,
            after,
            first,
            |query| app.governance().list_policies_by_created_at(sub, query)
        )
    }

    async fn approval_process(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<ApprovalProcess>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            ApprovalProcess,
            ctx,
            app.governance().find_approval_process_by_id(sub, id)
        )
    }

    async fn approval_processes(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<ApprovalProcessesByCreatedAtCursor, ApprovalProcess, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            ApprovalProcessesByCreatedAtCursor,
            ApprovalProcess,
            ctx,
            after,
            first,
            |query| app.governance().list_approval_processes(sub, query)
        )
    }

    async fn document(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Document>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Document, ctx, app.documents().find_by_id(sub, id))
    }

    async fn ledger_account(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<LedgerAccount>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            LedgerAccount,
            ctx,
            app.accounting()
                .find_ledger_account_by_id(sub, CHART_REF.0, id)
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

        let csv = AccountingCsv::from(csv);
        Ok(LedgerAccountCsvCreatePayload::from(csv))
    }

    async fn ledger_account_by_code(
        &self,
        ctx: &Context<'_>,
        code: String,
    ) -> async_graphql::Result<Option<LedgerAccount>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            LedgerAccount,
            ctx,
            app.accounting()
                .find_ledger_account_by_code(sub, CHART_REF.0, code)
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
            ctx,
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
            ctx,
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
            ctx,
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
            .find_by_reference_with_sub(sub, CHART_REF.0)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {}", CHART_REF.0));
        Ok(ChartOfAccounts::from(chart))
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

    async fn realtime_price(&self, ctx: &Context<'_>) -> async_graphql::Result<RealtimePrice> {
        let app = ctx.data_unchecked::<LanaApp>();
        let usd_cents_per_btc = app.price().usd_cents_per_btc().await?;
        Ok(usd_cents_per_btc.into())
    }

    async fn report(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Report>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let report = app.reports().find_by_id(sub, id).await?;
        Ok(report.map(Report::from))
    }

    async fn reports(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Report>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let users = app.reports().list_reports(sub).await?;
        Ok(users.into_iter().map(Report::from).collect())
    }

    async fn audit(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<AuditCursor, AuditEntry>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .list_audit(
                        sub,
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after.map(lana_app::audit::AuditCursor::from),
                        },
                    )
                    .await?;

                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entry| {
                        let cursor = AuditCursor::from(&entry);
                        Edge::new(cursor, AuditEntry::from(entry))
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn deposit_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<DepositModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .deposits()
            .get_chart_of_accounts_integration_config(sub)
            .await?;
        Ok(config.map(DepositModuleConfig::from))
    }

    async fn credit_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<CreditModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .credit()
            .chart_of_accounts_integrations()
            .get_config(sub)
            .await?;
        Ok(config.map(CreditModuleConfig::from))
    }

    async fn balance_sheet_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<BalanceSheetModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .accounting()
            .balance_sheets()
            .get_chart_of_accounts_integration_config(sub, BALANCE_SHEET_NAME.to_string())
            .await?;
        Ok(config.map(BalanceSheetModuleConfig::from))
    }

    async fn profit_and_loss_statement_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ProfitAndLossStatementModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .accounting()
            .profit_and_loss()
            .get_chart_of_accounts_integration_config(
                sub,
                PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
            )
            .await?;
        Ok(config.map(ProfitAndLossStatementModuleConfig::from))
    }

    async fn accounting_csvs_for_ledger_account_id(
        &self,
        ctx: &Context<'_>,
        ledger_account_id: UUID,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<AccountingCsvsByCreatedAtCursor, AccountingCsv, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            AccountingCsvsByCreatedAtCursor,
            AccountingCsv,
            ctx,
            after,
            first,
            |query| app.accounting().csvs().list_for_ledger_account_id(
                sub,
                query,
                ledger_account_id
            )
        )
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn customer_document_attach(
        &self,
        ctx: &Context<'_>,
        input: DocumentCreateInput,
    ) -> async_graphql::Result<DocumentCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let mut file = input.file.value(ctx)?;
        let mut data = Vec::new();
        file.content.read_to_end(&mut data)?;

        exec_mutation!(
            DocumentCreatePayload,
            Document,
            ctx,
            app.documents()
                .create(sub, data, input.customer_id, file.filename)
        )
    }

    pub async fn sumsub_permalink_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubPermalinkCreateInput,
    ) -> async_graphql::Result<SumsubPermalinkCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let permalink = app
            .applicants()
            .create_permalink(sub, input.customer_id)
            .await?;
        Ok(SumsubPermalinkCreatePayload { url: permalink.url })
    }

    async fn user_create(
        &self,
        ctx: &Context<'_>,
        input: UserCreateInput,
    ) -> async_graphql::Result<UserCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            UserCreatePayload,
            User,
            ctx,
            app.access().users().create_user(sub, input.email)
        )
    }

    async fn user_update_role(
        &self,
        ctx: &Context<'_>,
        input: UserUpdateRoleInput,
    ) -> async_graphql::Result<UserUpdateRolePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let UserUpdateRoleInput { id, role_id } = input;
        exec_mutation!(
            UserUpdateRolePayload,
            User,
            ctx,
            app.access().update_role_of_user(sub, id, role_id)
        )
    }

    async fn user_revoke_role(
        &self,
        ctx: &Context<'_>,
        input: UserRevokeRoleInput,
    ) -> async_graphql::Result<UserRevokeRolePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            UserRevokeRolePayload,
            User,
            ctx,
            app.access().revoke_role_from_user(sub, input.id)
        )
    }

    async fn role_create(
        &self,
        ctx: &Context<'_>,
        input: RoleCreateInput,
    ) -> async_graphql::Result<RoleCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let RoleCreateInput {
            name,
            permission_set_ids,
        } = input;
        exec_mutation!(
            RoleCreatePayload,
            Role,
            ctx,
            app.access().create_role(sub, name, permission_set_ids)
        )
    }

    async fn role_add_permission_sets(
        &self,
        ctx: &Context<'_>,
        input: RoleAddPermissionSetsInput,
    ) -> async_graphql::Result<RoleAddPermissionSetsPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        exec_mutation!(
            RoleAddPermissionSetsPayload,
            Role,
            ctx,
            app.access()
                .add_permission_sets_to_role(sub, input.role_id, input.permission_set_ids)
        )
    }

    async fn role_remove_permission_sets(
        &self,
        ctx: &Context<'_>,
        input: RoleRemovePermissionSetsInput,
    ) -> async_graphql::Result<RoleRemovePermissionSetsPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        exec_mutation!(
            RoleRemovePermissionSetsPayload,
            Role,
            ctx,
            app.access().remove_permission_sets_from_role(
                sub,
                input.role_id,
                input.permission_set_ids
            )
        )
    }

    async fn customer_create(
        &self,
        ctx: &Context<'_>,
        input: CustomerCreateInput,
    ) -> async_graphql::Result<CustomerCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerCreatePayload,
            Customer,
            ctx,
            app.customers()
                .create(sub, input.email, input.telegram_id, input.customer_type)
        )
    }

    async fn customer_telegram_id_update(
        &self,
        ctx: &Context<'_>,
        input: CustomerTelegramIdUpdateInput,
    ) -> async_graphql::Result<CustomerTelegramIdUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerTelegramIdUpdatePayload,
            Customer,
            ctx,
            app.customers()
                .update_telegram_id(sub, input.customer_id, input.telegram_id)
        )
    }

    async fn customer_email_update(
        &self,
        ctx: &Context<'_>,
        input: CustomerEmailUpdateInput,
    ) -> async_graphql::Result<CustomerEmailUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerEmailUpdatePayload,
            Customer,
            ctx,
            app.customers()
                .update_email(sub, input.customer_id, input.email)
        )
    }

    async fn deposit_module_configure(
        &self,
        ctx: &Context<'_>,
        input: DepositModuleConfigureInput,
    ) -> async_graphql::Result<DepositModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let chart = loader
            .load_one(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {:?}", CHART_REF));

        let config_values = lana_app::deposit::ChartOfAccountsIntegrationConfig::builder()
            .chart_of_accounts_id(chart.id)
            .chart_of_accounts_individual_deposit_accounts_parent_code(
                input
                    .chart_of_accounts_individual_deposit_accounts_parent_code
                    .parse()?,
            )
            .chart_of_accounts_government_entity_deposit_accounts_parent_code(
                input
                    .chart_of_accounts_government_entity_deposit_accounts_parent_code
                    .parse()?,
            )
            .chart_of_account_private_company_deposit_accounts_parent_code(
                input
                    .chart_of_account_private_company_deposit_accounts_parent_code
                    .parse()?,
            )
            .chart_of_account_bank_deposit_accounts_parent_code(
                input
                    .chart_of_account_bank_deposit_accounts_parent_code
                    .parse()?,
            )
            .chart_of_account_financial_institution_deposit_accounts_parent_code(
                input
                    .chart_of_account_financial_institution_deposit_accounts_parent_code
                    .parse()?,
            )
            .chart_of_account_non_domiciled_individual_deposit_accounts_parent_code(
                input
                    .chart_of_account_non_domiciled_individual_deposit_accounts_parent_code
                    .parse()?,
            )
            .chart_of_accounts_omnibus_parent_code(
                input.chart_of_accounts_omnibus_parent_code.parse()?,
            )
            .build()?;
        let config = app
            .deposits()
            .set_chart_of_accounts_integration_config(sub, chart.as_ref(), config_values)
            .await?;
        Ok(DepositModuleConfigurePayload::from(
            DepositModuleConfig::from(config),
        ))
    }

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
            ctx,
            app.accounting().execute_manual_transaction(
                sub,
                CHART_REF.0,
                input.reference,
                input.description,
                input.effective.map(|ts| ts.into_inner()),
                entries
            )
        )
    }

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

    async fn terms_template_create(
        &self,
        ctx: &Context<'_>,
        input: TermsTemplateCreateInput,
    ) -> async_graphql::Result<TermsTemplateCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let term_values = lana_app::terms::TermValues::builder()
            .annual_rate(input.annual_rate)
            .accrual_interval(input.accrual_interval)
            .accrual_cycle_interval(input.accrual_cycle_interval)
            .one_time_fee_rate(input.one_time_fee_rate)
            .duration(input.duration)
            .interest_due_duration_from_accrual(input.interest_due_duration_from_accrual)
            .obligation_overdue_duration_from_due(input.obligation_overdue_duration_from_due)
            .obligation_liquidation_duration_from_due(
                input.obligation_liquidation_duration_from_due,
            )
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;

        exec_mutation!(
            TermsTemplateCreatePayload,
            TermsTemplate,
            ctx,
            app.credit()
                .terms_templates()
                .create_terms_template(sub, input.name, term_values)
        )
    }

    async fn terms_template_update(
        &self,
        ctx: &Context<'_>,
        input: TermsTemplateUpdateInput,
    ) -> async_graphql::Result<TermsTemplateUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let term_values = lana_app::terms::TermValues::builder()
            .annual_rate(input.annual_rate)
            .accrual_interval(input.accrual_interval)
            .accrual_cycle_interval(input.accrual_cycle_interval)
            .one_time_fee_rate(input.one_time_fee_rate)
            .duration(input.duration)
            .interest_due_duration_from_accrual(input.interest_due_duration_from_accrual)
            .obligation_overdue_duration_from_due(input.obligation_overdue_duration_from_due)
            .obligation_liquidation_duration_from_due(
                input.obligation_liquidation_duration_from_due,
            )
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;
        exec_mutation!(
            TermsTemplateUpdatePayload,
            TermsTemplate,
            ctx,
            app.credit().terms_templates().update_term_values(
                sub,
                TermsTemplateId::from(input.id),
                term_values
            )
        )
    }

    async fn credit_module_configure(
        &self,
        ctx: &Context<'_>,
        input: CreditModuleConfigureInput,
    ) -> async_graphql::Result<CreditModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let chart = loader
            .load_one(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {:?}", CHART_REF));

        let CreditModuleConfigureInput {
            chart_of_account_facility_omnibus_parent_code,
            chart_of_account_collateral_omnibus_parent_code,
            chart_of_account_facility_parent_code,
            chart_of_account_collateral_parent_code,
            chart_of_account_interest_income_parent_code,
            chart_of_account_fee_income_parent_code,

            chart_of_account_short_term_individual_disbursed_receivable_parent_code,
            chart_of_account_short_term_government_entity_disbursed_receivable_parent_code,
            chart_of_account_short_term_private_company_disbursed_receivable_parent_code,
            chart_of_account_short_term_bank_disbursed_receivable_parent_code,
            chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code,
            chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code,

            chart_of_account_long_term_individual_disbursed_receivable_parent_code,
            chart_of_account_long_term_government_entity_disbursed_receivable_parent_code,
            chart_of_account_long_term_private_company_disbursed_receivable_parent_code,
            chart_of_account_long_term_bank_disbursed_receivable_parent_code,
            chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code,
            chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code,

            chart_of_account_short_term_individual_interest_receivable_parent_code,
            chart_of_account_short_term_government_entity_interest_receivable_parent_code,
            chart_of_account_short_term_private_company_interest_receivable_parent_code,
            chart_of_account_short_term_bank_interest_receivable_parent_code,
            chart_of_account_short_term_financial_institution_interest_receivable_parent_code,
            chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code,

            chart_of_account_long_term_individual_interest_receivable_parent_code,
            chart_of_account_long_term_government_entity_interest_receivable_parent_code,
            chart_of_account_long_term_private_company_interest_receivable_parent_code,
            chart_of_account_long_term_bank_interest_receivable_parent_code,
            chart_of_account_long_term_financial_institution_interest_receivable_parent_code,
            chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code,

            chart_of_account_overdue_individual_disbursed_receivable_parent_code,
            chart_of_account_overdue_government_entity_disbursed_receivable_parent_code,
            chart_of_account_overdue_private_company_disbursed_receivable_parent_code,
            chart_of_account_overdue_bank_disbursed_receivable_parent_code,
            chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code,
            chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code,
        } = input;

        let config_values = lana_app::credit::ChartOfAccountsIntegrationConfig::builder()
            .chart_of_accounts_id(chart.id)
            .chart_of_account_facility_omnibus_parent_code(
                chart_of_account_facility_omnibus_parent_code
                    .parse()?,
            )
            .chart_of_account_collateral_omnibus_parent_code(
                chart_of_account_collateral_omnibus_parent_code
                    .parse()?,
            )
            .chart_of_account_facility_parent_code(
                chart_of_account_facility_parent_code.parse()?,
            )
            .chart_of_account_collateral_parent_code(
                chart_of_account_collateral_parent_code.parse()?,
            )
            .chart_of_account_interest_income_parent_code(
                chart_of_account_interest_income_parent_code.parse()?,
            )
            .chart_of_account_fee_income_parent_code(
                chart_of_account_fee_income_parent_code.parse()?,
            )
            .chart_of_account_short_term_individual_disbursed_receivable_parent_code(chart_of_account_short_term_individual_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_short_term_government_entity_disbursed_receivable_parent_code(chart_of_account_short_term_government_entity_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_short_term_private_company_disbursed_receivable_parent_code(chart_of_account_short_term_private_company_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_short_term_bank_disbursed_receivable_parent_code(chart_of_account_short_term_bank_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code(chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code(chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code(chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_long_term_individual_disbursed_receivable_parent_code(chart_of_account_long_term_individual_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_long_term_government_entity_disbursed_receivable_parent_code(chart_of_account_long_term_government_entity_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_long_term_private_company_disbursed_receivable_parent_code(chart_of_account_long_term_private_company_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_long_term_bank_disbursed_receivable_parent_code(chart_of_account_long_term_bank_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code(chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code(chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code(chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code.parse()?)

            .chart_of_account_short_term_individual_interest_receivable_parent_code(chart_of_account_short_term_individual_interest_receivable_parent_code.parse()?)
            .chart_of_account_short_term_government_entity_interest_receivable_parent_code(chart_of_account_short_term_government_entity_interest_receivable_parent_code.parse()?)
            .chart_of_account_short_term_private_company_interest_receivable_parent_code(chart_of_account_short_term_private_company_interest_receivable_parent_code.parse()?)
            .chart_of_account_short_term_bank_interest_receivable_parent_code(chart_of_account_short_term_bank_interest_receivable_parent_code.parse()?)
            .chart_of_account_short_term_financial_institution_interest_receivable_parent_code(chart_of_account_short_term_financial_institution_interest_receivable_parent_code.parse()?)
            .chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code(chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code.parse()?)
            .chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code(chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code.parse()?)
            .chart_of_account_long_term_individual_interest_receivable_parent_code(chart_of_account_long_term_individual_interest_receivable_parent_code.parse()?)
            .chart_of_account_long_term_government_entity_interest_receivable_parent_code(chart_of_account_long_term_government_entity_interest_receivable_parent_code.parse()?)
            .chart_of_account_long_term_private_company_interest_receivable_parent_code(chart_of_account_long_term_private_company_interest_receivable_parent_code.parse()?)
            .chart_of_account_long_term_bank_interest_receivable_parent_code(chart_of_account_long_term_bank_interest_receivable_parent_code.parse()?)
            .chart_of_account_long_term_financial_institution_interest_receivable_parent_code(chart_of_account_long_term_financial_institution_interest_receivable_parent_code.parse()?)
            .chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code(chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code.parse()?)
            .chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code(chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code.parse()?)

            .chart_of_account_overdue_individual_disbursed_receivable_parent_code(chart_of_account_overdue_individual_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_overdue_government_entity_disbursed_receivable_parent_code(chart_of_account_overdue_government_entity_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_overdue_private_company_disbursed_receivable_parent_code(chart_of_account_overdue_private_company_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_overdue_bank_disbursed_receivable_parent_code(chart_of_account_overdue_bank_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code(chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code(chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code.parse()?)
            .chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code(chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code.parse()?)

            .build()?;
        let config = app
            .credit()
            .chart_of_accounts_integrations()
            .set_config(sub, chart.as_ref(), config_values)
            .await?;
        Ok(CreditModuleConfigurePayload::from(
            CreditModuleConfig::from(config),
        ))
    }

    pub async fn credit_facility_create(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCreateInput,
    ) -> async_graphql::Result<CreditFacilityCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityCreateInput {
            facility,
            customer_id,
            disbursal_credit_account_id,
            terms,
        } = input;

        let credit_facility_term_values = lana_app::terms::TermValues::builder()
            .annual_rate(terms.annual_rate)
            .accrual_interval(terms.accrual_interval)
            .accrual_cycle_interval(terms.accrual_cycle_interval)
            .one_time_fee_rate(terms.one_time_fee_rate)
            .duration(terms.duration)
            .interest_due_duration_from_accrual(terms.interest_due_duration_from_accrual)
            .obligation_overdue_duration_from_due(terms.obligation_overdue_duration_from_due)
            .obligation_liquidation_duration_from_due(
                terms.obligation_liquidation_duration_from_due,
            )
            .liquidation_cvl(terms.liquidation_cvl)
            .margin_call_cvl(terms.margin_call_cvl)
            .initial_cvl(terms.initial_cvl)
            .build()?;

        exec_mutation!(
            CreditFacilityCreatePayload,
            CreditFacility,
            ctx,
            app.credit().initiate(
                sub,
                customer_id,
                disbursal_credit_account_id,
                facility,
                credit_facility_term_values
            )
        )
    }

    pub async fn credit_facility_collateral_update(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCollateralUpdateInput,
    ) -> async_graphql::Result<CreditFacilityCollateralUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityCollateralUpdateInput {
            credit_facility_id,
            collateral,
            effective,
        } = input;
        exec_mutation!(
            CreditFacilityCollateralUpdatePayload,
            CreditFacility,
            ctx,
            app.credit()
                .update_collateral(sub, credit_facility_id, collateral, effective)
        )
    }

    pub async fn credit_facility_partial_payment(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityPartialPaymentPayload,
            CreditFacility,
            ctx,
            app.credit().record_payment(
                sub,
                input.credit_facility_id,
                input.amount,
                input.effective
            )
        )
    }

    pub async fn credit_facility_disbursal_initiate(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityDisbursalInitiateInput,
    ) -> async_graphql::Result<CreditFacilityDisbursalInitiatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityDisbursalInitiatePayload,
            CreditFacilityDisbursal,
            ctx,
            app.credit()
                .initiate_disbursal(sub, input.credit_facility_id.into(), input.amount)
        )
    }

    async fn credit_facility_complete(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCompleteInput,
    ) -> async_graphql::Result<CreditFacilityCompletePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityCompletePayload,
            CreditFacility,
            ctx,
            app.credit()
                .complete_facility(sub, input.credit_facility_id)
        )
    }

    async fn custodian_create(
        &self,
        ctx: &Context<'_>,
        input: CustodianCreateInput,
    ) -> async_graphql::Result<CustodianCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustodianCreatePayload,
            Custodian,
            ctx,
            app.custody()
                .create_custodian_config(sub, input.name().to_owned(), input.into())
        )
    }

    async fn committee_create(
        &self,
        ctx: &Context<'_>,
        input: CommitteeCreateInput,
    ) -> async_graphql::Result<CommitteeCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CommitteeCreatePayload,
            Committee,
            ctx,
            app.governance().create_committee(sub, input.name)
        )
    }

    async fn committee_add_user(
        &self,
        ctx: &Context<'_>,
        input: CommitteeAddUserInput,
    ) -> async_graphql::Result<CommitteeAddUserPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CommitteeAddUserPayload,
            Committee,
            ctx,
            app.governance()
                .add_member_to_committee(sub, input.committee_id, input.user_id)
        )
    }

    async fn committee_remove_user(
        &self,
        ctx: &Context<'_>,
        input: CommitteeRemoveUserInput,
    ) -> async_graphql::Result<CommitteeRemoveUserPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CommitteeRemoveUserPayload,
            Committee,
            ctx,
            app.governance()
                .remove_member_from_committee(sub, input.committee_id, input.user_id)
        )
    }

    async fn policy_assign_committee(
        &self,
        ctx: &Context<'_>,
        input: PolicyAssignCommitteeInput,
    ) -> async_graphql::Result<PolicyAssignCommitteePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            PolicyAssignCommitteePayload,
            Policy,
            ctx,
            app.governance().assign_committee_to_policy(
                sub,
                input.policy_id,
                input.committee_id,
                input.threshold
            )
        )
    }

    async fn approval_process_approve(
        &self,
        ctx: &Context<'_>,
        input: ApprovalProcessApproveInput,
    ) -> async_graphql::Result<ApprovalProcessApprovePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ApprovalProcessApprovePayload,
            ApprovalProcess,
            ctx,
            app.governance().approve_process(sub, input.process_id)
        )
    }

    async fn approval_process_deny(
        &self,
        ctx: &Context<'_>,
        input: ApprovalProcessDenyInput,
        reason: String,
    ) -> async_graphql::Result<ApprovalProcessDenyPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ApprovalProcessDenyPayload,
            ApprovalProcess,
            ctx,
            app.governance().deny_process(sub, input.process_id, reason)
        )
    }

    async fn document_download_link_generate(
        &self,
        ctx: &Context<'_>,
        input: DocumentDownloadLinksGenerateInput,
    ) -> async_graphql::Result<DocumentDownloadLinksGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        // not using macro here because DocumentDownloadLinksGeneratePayload is non standard
        let doc = app
            .documents()
            .generate_download_link(sub, input.document_id.into())
            .await?;
        Ok(DocumentDownloadLinksGeneratePayload::from(doc))
    }

    async fn document_delete(
        &self,
        ctx: &Context<'_>,
        input: DocumentDeleteInput,
    ) -> async_graphql::Result<DocumentDeletePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        // not using macro here because DocumentDeletePayload is non standard
        app.documents().delete(sub, input.document_id).await?;
        Ok(DocumentDeletePayload {
            deleted_document_id: input.document_id,
        })
    }

    async fn document_archive(
        &self,
        ctx: &Context<'_>,
        input: DocumentArchiveInput,
    ) -> async_graphql::Result<DocumentArchivePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DocumentArchivePayload,
            Document,
            ctx,
            app.documents().archive(sub, input.document_id)
        )
    }

    async fn report_create(&self, ctx: &Context<'_>) -> async_graphql::Result<ReportCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let report = app.reports().create(sub).await?;
        Ok(ReportCreatePayload::from(report))
    }

    async fn report_download_links_generate(
        &self,
        ctx: &Context<'_>,
        input: ReportDownloadLinksGenerateInput,
    ) -> async_graphql::Result<ReportDownloadLinksGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let links = app
            .reports()
            .generate_download_links(sub, input.report_id.into())
            .await?;
        Ok(ReportDownloadLinksGeneratePayload::from(links))
    }

    async fn chart_of_accounts_csv_import(
        &self,
        ctx: &Context<'_>,
        input: ChartOfAccountsCsvImportInput,
    ) -> async_graphql::Result<ChartOfAccountsCsvImportPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let ChartOfAccountsCsvImportInput { chart_id, file } = input;

        let mut file = file.value(ctx)?.content;

        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let res = app
            .accounting()
            .import_csv(sub, chart_id.into(), data, TRIAL_BALANCE_STATEMENT_NAME)
            .await?;

        Ok(ChartOfAccountsCsvImportPayload { success: res })
    }

    async fn balance_sheet_configure(
        &self,
        ctx: &Context<'_>,
        input: BalanceSheetModuleConfigureInput,
    ) -> async_graphql::Result<BalanceSheetModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let chart = loader
            .load_one(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {:?}", CHART_REF));

        let config_values = lana_app::balance_sheet::ChartOfAccountsIntegrationConfig::builder()
            .chart_of_accounts_id(chart.id)
            .chart_of_accounts_assets_code(input.chart_of_accounts_assets_code.parse()?)
            .chart_of_accounts_liabilities_code(input.chart_of_accounts_liabilities_code.parse()?)
            .chart_of_accounts_equity_code(input.chart_of_accounts_equity_code.parse()?)
            .chart_of_accounts_revenue_code(input.chart_of_accounts_revenue_code.parse()?)
            .chart_of_accounts_cost_of_revenue_code(
                input.chart_of_accounts_cost_of_revenue_code.parse()?,
            )
            .chart_of_accounts_expenses_code(input.chart_of_accounts_expenses_code.parse()?)
            .build()?;
        let config = app
            .accounting()
            .balance_sheets()
            .set_chart_of_accounts_integration_config(
                sub,
                BALANCE_SHEET_NAME.to_string(),
                chart.as_ref(),
                config_values,
            )
            .await?;
        Ok(BalanceSheetModuleConfigurePayload::from(
            BalanceSheetModuleConfig::from(config),
        ))
    }

    async fn profit_and_loss_statement_configure(
        &self,
        ctx: &Context<'_>,
        input: ProfitAndLossModuleConfigureInput,
    ) -> async_graphql::Result<ProfitAndLossStatementModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let chart = loader
            .load_one(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {:?}", CHART_REF));

        let config_values = lana_app::profit_and_loss::ChartOfAccountsIntegrationConfig::builder()
            .chart_of_accounts_id(chart.id)
            .chart_of_accounts_revenue_code(input.chart_of_accounts_revenue_code.parse()?)
            .chart_of_accounts_cost_of_revenue_code(
                input.chart_of_accounts_cost_of_revenue_code.parse()?,
            )
            .chart_of_accounts_expenses_code(input.chart_of_accounts_expenses_code.parse()?)
            .build()?;
        let config = app
            .accounting()
            .profit_and_loss()
            .set_chart_of_accounts_integration_config(
                sub,
                PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
                chart.as_ref(),
                config_values,
            )
            .await?;
        Ok(ProfitAndLossStatementModuleConfigurePayload::from(
            ProfitAndLossStatementModuleConfig::from(config),
        ))
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

        let csv = AccountingCsv::from(csv);
        Ok(LedgerAccountCsvCreatePayload::from(csv))
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
            .generate_download_link(sub, input.accounting_csv_id.into())
            .await?;

        let link = AccountingCsvDownloadLink::from(result);

        Ok(AccountingCsvDownloadLinkGeneratePayload::from(link))
    }
}
