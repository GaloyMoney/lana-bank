use async_graphql::{Context, Error, MergedObject, Object, Subscription, types::connection::*};

use admin_graphql_access::{AccessMutation, AccessQuery};
use admin_graphql_config::{ConfigMutation, ConfigQuery};
use admin_graphql_contracts::{ContractsMutation, ContractsQuery};
use admin_graphql_custody::{CustodyMutation, CustodyQuery};
use admin_graphql_documents::{DocumentsMutation, DocumentsQuery};
use admin_graphql_governance::{GovernanceMutation, GovernanceQuery};
use admin_graphql_reports::{ReportsMutation, ReportsQuery};
use admin_graphql_session::{SessionMutation, SessionQuery};

use std::io::Read;

use futures::StreamExt;
use futures::stream::Stream;
use obix::out::OutboxEventMarker;

use lana_app::accounting::CoreAccountingEvent;
use lana_app::credit::CoreCreditEvent;
use lana_app::customer::prospect_cursor::ProspectsByCreatedAtCursor;
use lana_app::price::CorePriceEvent;
use lana_app::report::CoreReportEvent;
use lana_app::{
    accounting_init::constants::{
        BALANCE_SHEET_NAME, PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME,
    },
    app::LanaApp,
    credit::LiquidationsByIdCursor,
};

use crate::primitives::*;

use super::{
    accounting::*, approval_process::*, audit::*, credit_facility::*, customer::*, deposit::*,
    loader::*, price::*, prospect::*, public_id::*, reports::*, withdrawal::*,
};

#[derive(MergedObject, Default)]
pub struct Query(
    pub AccessQuery,
    pub ConfigQuery,
    pub ContractsQuery,
    pub CustodyQuery,
    pub DocumentsQuery,
    pub GovernanceQuery,
    pub ReportsQuery,
    pub SessionQuery,
    pub BaseQuery,
);

#[derive(Default)]
pub struct BaseQuery;

#[Object]
impl BaseQuery {
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

    async fn customer_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<Customer>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Customer, ctx, app.customers().find_by_public_id(sub, id))
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
        let filter = DomainCustomersFilters {
            kyc_verification: filter.and_then(|f| f.kyc_verification),
            ..Default::default()
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

    async fn prospect(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Prospect>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Prospect,
            ProspectId,
            ctx,
            app.customers().find_prospect_by_id(sub, id)
        )
    }

    async fn prospect_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<Prospect>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Prospect,
            ctx,
            app.customers().find_prospect_by_public_id(sub, id)
        )
    }

    async fn prospects(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        stage: Option<lana_app::customer::ProspectStage>,
    ) -> async_graphql::Result<
        Connection<ProspectsByCreatedAtCursor, Prospect, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor_and_id!(
            ProspectsByCreatedAtCursor,
            Prospect,
            ProspectId,
            ctx,
            after,
            first,
            |query| app
                .customers()
                .list_prospects(sub, query, ListDirection::Descending, stage,)
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

    async fn credit_facility_proposal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacilityProposal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        maybe_fetch_one!(
            CreditFacilityProposal,
            ctx,
            app.credit().proposals().find_by_id(sub, id)
        )
    }

    async fn credit_facility_proposals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            CreditFacilityProposalsByCreatedAtCursor,
            CreditFacilityProposal,
            EmptyFields,
            EmptyFields,
        >,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            CreditFacilityProposalsByCreatedAtCursor,
            CreditFacilityProposal,
            ctx,
            after,
            first,
            |query| app.credit().proposals().list(sub, query)
        )
    }

    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<PendingCreditFacility>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        maybe_fetch_one!(
            PendingCreditFacility,
            ctx,
            app.credit().pending_credit_facilities().find_by_id(sub, id)
        )
    }

    async fn pending_credit_facilities(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            PendingCreditFacilitiesByCreatedAtCursor,
            PendingCreditFacility,
            EmptyFields,
            EmptyFields,
        >,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            PendingCreditFacilitiesByCreatedAtCursor,
            PendingCreditFacility,
            ctx,
            after,
            first,
            |query| app.credit().pending_credit_facilities().list(sub, query)
        )
    }

    async fn credit_facility_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<CreditFacility>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacility,
            ctx,
            app.credit().facilities().find_by_public_id(sub, id)
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
        let filter = DomainCreditFacilitiesFilters {
            status: filter.as_ref().and_then(|f| f.status),
            collateralization_state: filter.as_ref().and_then(|f| f.collateralization_state),
            customer_id: None,
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

    async fn disbursal_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<CreditFacilityDisbursal>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityDisbursal,
            ctx,
            app.credit().disbursals().find_by_public_id(sub, id)
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
        let filter = DisbursalsFilters::default();

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

    async fn liquidation(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Liquidation>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            Liquidation,
            ctx,
            app.credit().collaterals().find_liquidation_by_id(sub, id)
        )
    }

    async fn liquidations(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<LiquidationsByIdCursor, Liquidation, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            LiquidationsByIdCursor,
            Liquidation,
            ctx,
            after,
            first,
            |query| app.credit().collaterals().list_liquidations(sub, query)
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
            .descendant_account_sets_by_category(sub, CHART_REF.0, category.into())
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
            ctx,
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
            ctx,
            app.accounting()
                .find_fiscal_year_for_chart_by_year(sub, CHART_REF.0, &year)
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
            ctx,
            after,
            first,
            |query| app
                .accounting()
                .list_fiscal_years_for_chart(sub, CHART_REF.0, query,)
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

    async fn realtime_price(&self, ctx: &Context<'_>) -> async_graphql::Result<RealtimePrice> {
        let app = ctx.data_unchecked::<LanaApp>();
        let usd_cents_per_btc = app.price().usd_cents_per_btc().await;
        Ok(usd_cents_per_btc.into())
    }

    async fn audit(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        subject: Option<AuditSubjectId>,
        authorized: Option<bool>,
        object: Option<String>,
        action: Option<String>,
    ) -> async_graphql::Result<Connection<AuditCursor, AuditEntry>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let subject_filter: Option<String> = subject.map(String::from);
        let authorized_filter = authorized;
        let object_filter = object;
        let action_filter = action;
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
                        subject_filter.clone(),
                        authorized_filter,
                        object_filter.clone(),
                        action_filter.clone(),
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

    async fn audit_subjects(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<AuditSubjectId>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .list_audit_subjects(sub)
            .await?
            .into_iter()
            .map(AuditSubjectId::from)
            .collect())
    }

    async fn public_id_target(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<PublicIdTarget>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let Some(public_id) = app.public_ids().find_by_id(id).await? else {
            return Ok(None);
        };

        let res = match public_id.target_type.as_str() {
            "customer" => self
                .customer(ctx, public_id.target_id.into())
                .await?
                .map(PublicIdTarget::Customer),
            "deposit_account" => self
                .deposit_account(ctx, public_id.target_id.into())
                .await?
                .map(PublicIdTarget::DepositAccount),
            "deposit" => self
                .deposit(ctx, public_id.target_id.into())
                .await?
                .map(PublicIdTarget::Deposit),
            "withdrawal" => self
                .withdrawal(ctx, public_id.target_id.into())
                .await?
                .map(PublicIdTarget::Withdrawal),
            "credit_facility" => self
                .credit_facility(ctx, public_id.target_id.into())
                .await?
                .map(PublicIdTarget::CreditFacility),
            "disbursal" => self
                .disbursal(ctx, public_id.target_id.into())
                .await?
                .map(PublicIdTarget::CreditFacilityDisbursal),
            "prospect" => self
                .prospect(ctx, public_id.target_id.into())
                .await?
                .map(PublicIdTarget::Prospect),
            _ => None,
        };
        Ok(res)
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

#[derive(MergedObject, Default)]
pub struct Mutation(
    pub AccessMutation,
    pub ConfigMutation,
    pub ContractsMutation,
    pub CustodyMutation,
    pub DocumentsMutation,
    pub GovernanceMutation,
    pub ReportsMutation,
    pub SessionMutation,
    pub BaseMutation,
);

#[derive(Default)]
pub struct BaseMutation;

#[Object]
impl BaseMutation {
    async fn prospect_create(
        &self,
        ctx: &Context<'_>,
        input: ProspectCreateInput,
    ) -> async_graphql::Result<ProspectCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ProspectCreatePayload,
            Prospect,
            ProspectId,
            ctx,
            app.customers().create_prospect(
                sub,
                input.email,
                input.telegram_handle,
                input.customer_type
            )
        )
    }

    async fn prospect_close(
        &self,
        ctx: &Context<'_>,
        input: ProspectCloseInput,
    ) -> async_graphql::Result<ProspectClosePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            ProspectClosePayload,
            Prospect,
            ProspectId,
            ctx,
            app.customers().close_prospect(sub, input.prospect_id)
        )
    }

    async fn prospect_convert(
        &self,
        ctx: &Context<'_>,
        input: ProspectConvertInput,
    ) -> async_graphql::Result<ProspectConvertPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let require_verified = app
            .exposed_domain_configs()
            .get::<lana_app::deposit::RequireVerifiedCustomerForAccount>(sub)
            .await?
            .value();
        if require_verified {
            return Err(Error::new(
                "Manual conversion is only available when 'Require verified customer for account' is disabled",
            ));
        }
        exec_mutation!(
            ProspectConvertPayload,
            Customer,
            ctx,
            app.customers().convert_prospect(sub, input.prospect_id)
        )
    }

    async fn customer_telegram_handle_update(
        &self,
        ctx: &Context<'_>,
        input: CustomerTelegramHandleUpdateInput,
    ) -> async_graphql::Result<CustomerTelegramHandleUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerTelegramHandleUpdatePayload,
            Customer,
            ctx,
            app.customers()
                .update_telegram_handle(sub, input.customer_id, input.telegram_handle)
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

    pub async fn credit_facility_proposal_create(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityProposalCreateInput,
    ) -> async_graphql::Result<CreditFacilityProposalCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityProposalCreateInput {
            facility,
            customer_id,
            terms,
            custodian_id,
        } = input;

        let credit_facility_term_values = lana_app::terms::TermValues::builder()
            .annual_rate(terms.annual_rate)
            .accrual_interval(terms.accrual_interval)
            .accrual_cycle_interval(terms.accrual_cycle_interval)
            .one_time_fee_rate(terms.one_time_fee_rate)
            .disbursal_policy(terms.disbursal_policy)
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
            CreditFacilityProposalCreatePayload,
            CreditFacilityProposal,
            ctx,
            app.create_facility_proposal(
                sub,
                customer_id,
                facility,
                credit_facility_term_values,
                custodian_id
            )
        )
    }

    pub async fn credit_facility_proposal_customer_approval_conclude(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityProposalCustomerApprovalConcludeInput,
    ) -> async_graphql::Result<CreditFacilityProposalCustomerApprovalConcludePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityProposalCustomerApprovalConcludeInput {
            credit_facility_proposal_id,
            approved,
        } = input;

        exec_mutation!(
            CreditFacilityProposalCustomerApprovalConcludePayload,
            CreditFacilityProposal,
            ctx,
            app.credit().proposals().conclude_customer_approval(
                sub,
                credit_facility_proposal_id,
                approved
            )
        )
    }

    pub async fn collateral_update(
        &self,
        ctx: &Context<'_>,
        input: CollateralUpdateInput,
    ) -> async_graphql::Result<CollateralUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CollateralUpdateInput {
            collateral_id,
            collateral,
            effective,
        } = input;
        exec_mutation!(
            CollateralUpdatePayload,
            Collateral,
            ctx,
            app.credit().collaterals().update_collateral_by_id(
                sub,
                collateral_id.into(),
                collateral,
                effective.into()
            )
        )
    }

    pub async fn credit_facility_partial_payment_record(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentRecordInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentRecordPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityPartialPaymentRecordPayload,
            CreditFacility,
            ctx,
            app.record_payment(sub, input.credit_facility_id, input.amount,)
        )
    }

    pub async fn credit_facility_partial_payment_with_date_record(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentWithDateRecordInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentRecordPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityPartialPaymentRecordPayload,
            CreditFacility,
            ctx,
            app.record_payment_with_date(
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

    async fn collateral_record_sent_to_liquidation(
        &self,
        ctx: &Context<'_>,
        input: CollateralRecordSentToLiquidationInput,
    ) -> async_graphql::Result<CollateralRecordSentToLiquidationPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CollateralRecordSentToLiquidationPayload,
            Collateral,
            ctx,
            app.credit()
                .collaterals()
                .record_collateral_update_via_liquidation(
                    sub,
                    input.collateral_id.into(),
                    input.amount
                )
        )
    }

    async fn collateral_record_proceeds_from_liquidation(
        &self,
        ctx: &Context<'_>,
        input: CollateralRecordProceedsFromLiquidationInput,
    ) -> async_graphql::Result<CollateralRecordProceedsFromLiquidationPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CollateralRecordProceedsFromLiquidationPayload,
            Collateral,
            ctx,
            app.credit()
                .collaterals()
                .record_proceeds_received_and_liquidation_completed(
                    sub,
                    input.collateral_id.into(),
                    input.amount
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
            ChartId,
            ctx,
            app.accounting()
                .import_csv(sub, CHART_REF.0, data, TRIAL_BALANCE_STATEMENT_NAME)
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
            FiscalYearId,
            ctx,
            app.accounting()
                .init_fiscal_year_for_chart(sub, CHART_REF.0, input.opened_as_of)
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
            FiscalYearId,
            ctx,
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
            FiscalYearId,
            ctx,
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
            FiscalYearId,
            ctx,
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
            ChartId,
            ctx,
            app.accounting().add_root_node(
                sub,
                CHART_REF.0,
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
            ChartId,
            ctx,
            app.accounting().add_child_node(
                sub,
                CHART_REF.0,
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
            ChartId,
            ctx,
            app.accounting().import_csv_with_base_config(
                sub,
                CHART_REF.0,
                data,
                input.base_config.try_into()?,
                BALANCE_SHEET_NAME,
                PROFIT_AND_LOSS_STATEMENT_NAME,
                TRIAL_BALANCE_STATEMENT_NAME
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

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn pending_credit_facility_collateralization_updated(
        &self,
        ctx: &Context<'_>,
        pending_credit_facility_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = PendingCreditFacilityCollateralizationPayload>>
    {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let pending_credit_facility_id = PendingCreditFacilityId::from(pending_credit_facility_id);

        app.credit()
            .pending_credit_facilities()
            .find_by_id(sub, pending_credit_facility_id)
            .await?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |message| async move {
            let payload = message.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::PendingCreditFacilityCollateralizationChanged { entity }
                    if entity.id == pending_credit_facility_id =>
                {
                    let collateralization = &entity.collateralization;
                    Some(PendingCreditFacilityCollateralizationPayload {
                        pending_credit_facility_id,
                        update: PendingCreditFacilityCollateralizationUpdated {
                            state: collateralization.state,
                            collateral: collateralization.collateral.expect("collateral must be set for PendingCreditFacilityCollateralizationChanged"),
                            price: collateralization.price_at_state_change.expect("price must be set for PendingCreditFacilityCollateralizationChanged").into_inner(),
                            recorded_at: message.recorded_at.into(),
                            effective: message.recorded_at.date_naive().into(),
                        },
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn pending_credit_facility_completed(
        &self,
        ctx: &Context<'_>,
        pending_credit_facility_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = PendingCreditFacilityCompletedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let pending_credit_facility_id = PendingCreditFacilityId::from(pending_credit_facility_id);

        app.credit()
            .pending_credit_facilities()
            .find_by_id(sub, pending_credit_facility_id)
            .await?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::PendingCreditFacilityCompleted { entity }
                    if entity.id == pending_credit_facility_id =>
                {
                    Some(PendingCreditFacilityCompletedPayload {
                        pending_credit_facility_id,
                        update: PendingCreditFacilityCompleted {
                            status: entity.status,
                            recorded_at: entity.completed_at?.into(),
                        },
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn credit_facility_proposal_concluded(
        &self,
        ctx: &Context<'_>,
        credit_facility_proposal_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = CreditFacilityProposalConcludedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let credit_facility_proposal_id =
            CreditFacilityProposalId::from(credit_facility_proposal_id);

        app.credit()
            .proposals()
            .find_by_id(sub, credit_facility_proposal_id)
            .await?
            .ok_or_else(|| Error::new("Credit facility proposal not found"))?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::FacilityProposalConcluded { entity }
                    if entity.id == credit_facility_proposal_id =>
                {
                    Some(CreditFacilityProposalConcludedPayload {
                        credit_facility_proposal_id,
                        status: entity.status,
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn credit_facility_collateralization_updated(
        &self,
        ctx: &Context<'_>,
        credit_facility_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = CreditFacilityCollateralizationPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let credit_facility_id = CreditFacilityId::from(credit_facility_id);

        app.credit()
            .facilities()
            .find_by_id(sub, credit_facility_id)
            .await?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |message| async move {
            let payload = message.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::FacilityCollateralizationChanged { entity }
                    if entity.id == credit_facility_id =>
                {
                    let collateralization = &entity.collateralization;
                    Some(CreditFacilityCollateralizationPayload {
                        credit_facility_id,
                        update: CreditFacilityCollateralizationUpdated {
                            state: collateralization.state,
                            collateral: collateralization.collateral,
                            outstanding_interest: collateralization.outstanding.interest,
                            outstanding_disbursal: collateralization.outstanding.disbursed,
                            recorded_at: message.recorded_at.into(),
                            effective: message.recorded_at.date_naive().into(),
                            price: collateralization.price_at_state_change.into_inner(),
                        },
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn ledger_account_csv_export_uploaded(
        &self,
        ctx: &Context<'_>,
        ledger_account_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = LedgerAccountCsvExportUploadedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let ledger_account_id = LedgerAccountId::from(ledger_account_id);

        app.accounting()
            .find_ledger_account_by_id(sub, CHART_REF.0, ledger_account_id)
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
                        document_id: UUID::from(*id),
                    })
                }
                _ => None,
            }
        });

        Ok(updates)
    }

    async fn realtime_price_updated(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<impl Stream<Item = RealtimePrice>> {
        let app = ctx.data_unchecked::<LanaApp>();

        let stream = app.outbox().listen_ephemeral();
        let updates = stream.filter_map(move |event| async move {
            let event: &CorePriceEvent = event.payload.as_event()?;
            match event {
                CorePriceEvent::PriceUpdated { price, .. } => Some(RealtimePrice::from(*price)),
            }
        });

        Ok(updates)
    }

    async fn report_run_updated(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<impl Stream<Item = ReportRunUpdatedPayload>> {
        let app = ctx.data_unchecked::<LanaApp>();

        let stream = app.outbox().listen_ephemeral();
        let updates = stream.filter_map(move |event| async move {
            let event: &CoreReportEvent = event.payload.as_event()?;
            match event {
                CoreReportEvent::ReportRunCreated { entity }
                | CoreReportEvent::ReportRunStateUpdated { entity } => {
                    Some(ReportRunUpdatedPayload {
                        report_run_id: UUID::from(entity.id),
                    })
                }
            }
        });

        Ok(updates)
    }
}
