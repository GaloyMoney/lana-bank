use async_graphql::{Context, Error, Object, Subscription, types::connection::*};

use std::io::Read;

use futures::StreamExt;
use futures::stream::Stream;
use obix::out::OutboxEventMarker;

use lana_app::accounting::CoreAccountingEvent;
use lana_app::credit::CoreCreditEvent;
use lana_app::customer::CoreCustomerEvent;
use lana_app::price::CorePriceEvent;
use lana_app::{
    accounting_init::constants::{
        BALANCE_SHEET_NAME, PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME,
    },
    app::LanaApp,
    credit::LiquidationsByIdCursor,
};

use crate::primitives::*;

use super::{
    access::*, accounting::*, approval_process::*, audit::*, balance_sheet_config::*, committee::*,
    contract_creation::*, credit_config::*, credit_facility::*, custody::*, customer::*,
    dashboard::*, deposit::*, deposit_config::*, document::*, domain_config::*,
    fiscal_year_config::*, loader::*, me::*, policy::*, price::*, profit_and_loss_config::*,
    public_id::*, reports::*, sumsub::*, terms_template::*, withdrawal::*,
};

pub struct Query;

#[Object]
impl Query {
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<MeUser> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let user = Arc::new(app.access().users().find_for_subject(sub).await?);
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        loader.feed_one(user.id, User::from(user.clone())).await;
        Ok(MeUser::from(user))
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
        let (filter_field, status) = match filter {
            Some(filter) => (Some(filter.field), filter.kyc_verification),
            None => (None, None),
        };
        let filter = match filter_field {
            None => DomainCustomersFilter::NoFilter,
            Some(CustomersFilterBy::AccountKycVerification) => {
                let status = status.ok_or(CustomerError::MissingValueForFilterField(
                    "kyc_verification".to_string(),
                ))?;
                DomainCustomersFilter::WithKycVerification(status)
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
        let (filter_field, status, collateralization_state) = match filter {
            Some(filter) => (
                Some(filter.field),
                filter.status,
                filter.collateralization_state,
            ),
            None => (None, None, None),
        };
        let filter = match filter_field {
            None => DomainCreditFacilitiesFilter::NoFilter,
            Some(CreditFacilitiesFilterBy::Status) => {
                let status = status.ok_or(CreditFacilityError::MissingValueForFilterField(
                    "status".to_string(),
                ))?;
                DomainCreditFacilitiesFilter::WithStatus(status)
            }
            Some(CreditFacilitiesFilterBy::CollateralizationState) => {
                let collateralization_state = collateralization_state.ok_or(
                    CreditFacilityError::MissingValueForFilterField(
                        "collateralization_state".to_string(),
                    ),
                )?;
                DomainCreditFacilitiesFilter::WithCollateralizationState(collateralization_state)
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
        let filter = DisbursalsFilter::NoFilter;

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
            app.credit().liquidations().find_by_id(sub, id)
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
            |query| app.credit().liquidations().list(sub, query)
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

    async fn customer_document(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CustomerDocument>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CustomerDocument,
            CustomerDocumentId,
            ctx,
            app.customers().find_customer_document_by_id(sub, id)
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

    async fn domain_configs(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DomainConfigsByKeyCursor, DomainConfig, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DomainConfigsByKeyCursor,
            DomainConfig,
            ctx,
            after,
            first,
            |query| app.exposed_domain_configs().list(sub, query)
        )
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
            _ => unreachable!(),
        };
        Ok(res)
    }

    async fn loan_agreement(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<LoanAgreement>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let agreement = app.contract_creation().find_by_id(sub, id).await?;
        Ok(agreement.map(LoanAgreement::from))
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

    async fn report_runs(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<ReportRunsByCreatedAtCursor, ReportRun, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            ReportRunsByCreatedAtCursor,
            ReportRun,
            ctx,
            after,
            first,
            |query| app.reports().list_report_runs(sub, query)
        )
    }

    async fn report_run(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<ReportRun>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(ReportRun, ctx, app.reports().find_report_run_by_id(sub, id))
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn customer_document_attach(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentCreateInput,
    ) -> async_graphql::Result<CustomerDocumentCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let mut file = input.file.value(ctx)?;
        let mut data = Vec::new();
        file.content.read_to_end(&mut data)?;
        exec_mutation!(
            CustomerDocumentCreatePayload,
            CustomerDocument,
            CustomerDocumentId,
            ctx,
            app.customers().create_document(
                sub,
                input.customer_id,
                data,
                file.filename,
                file.content_type
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
            )
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
            .create_permalink(
                sub,
                lana_app::primitives::CustomerId::from(input.customer_id),
            )
            .await?;
        Ok(SumsubPermalinkCreatePayload { url: permalink.url })
    }

    /// ⚠️ TEST ONLY: Creates a complete test applicant for Sumsub integration testing.
    /// This method is behind a compilation flag and should only be used in test environments.
    #[cfg(feature = "sumsub-testing")]
    pub async fn sumsub_test_applicant_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubTestApplicantCreateInput,
    ) -> async_graphql::Result<SumsubTestApplicantCreatePayload> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let applicant_id = app
            .applicants()
            .create_complete_test_applicant(lana_app::primitives::CustomerId::from(
                input.customer_id,
            ))
            .await?;
        Ok(SumsubTestApplicantCreatePayload { applicant_id })
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
            app.access().create_user(sub, input.email, input.role_id)
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

    async fn domain_config_update(
        &self,
        ctx: &Context<'_>,
        input: DomainConfigUpdateInput,
    ) -> async_graphql::Result<DomainConfigUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DomainConfigUpdatePayload,
            DomainConfig,
            ctx,
            app.exposed_domain_configs().update_from_json(
                sub,
                input.domain_config_id,
                input.value.into_inner(),
            )
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
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {CHART_REF:?}"));

        let DepositModuleConfigureInput {
            chart_of_accounts_omnibus_parent_code,
            chart_of_accounts_individual_deposit_accounts_parent_code,
            chart_of_accounts_government_entity_deposit_accounts_parent_code,
            chart_of_account_private_company_deposit_accounts_parent_code,
            chart_of_account_bank_deposit_accounts_parent_code,
            chart_of_account_financial_institution_deposit_accounts_parent_code,
            chart_of_account_non_domiciled_individual_deposit_accounts_parent_code,
            chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            chart_of_account_frozen_bank_deposit_accounts_parent_code,
            chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code,
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
            chart_of_account_non_domiciled_individual_deposit_accounts_parent_code:
                chart_of_account_non_domiciled_individual_deposit_accounts_parent_code.parse()?,
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
            chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code:
                chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code
                    .parse()?,
            chart_of_accounts_omnibus_parent_code: chart_of_accounts_omnibus_parent_code.parse()?,
        };

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
            .disbursal_policy(input.disbursal_policy)
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
            .disbursal_policy(input.disbursal_policy)
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
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {CHART_REF:?}"));

        let CreditModuleConfigureInput {
            chart_of_account_facility_omnibus_parent_code,
            chart_of_account_collateral_omnibus_parent_code,
            chart_of_account_liquidation_proceeds_omnibus_parent_code,
            chart_of_account_payments_made_omnibus_parent_code,
            chart_of_account_interest_added_to_obligations_omnibus_parent_code,
            chart_of_account_facility_parent_code,
            chart_of_account_collateral_parent_code,
            chart_of_account_collateral_in_liquidation_parent_code,
            chart_of_account_interest_income_parent_code,
            chart_of_account_fee_income_parent_code,
            chart_of_account_payment_holding_parent_code,
            chart_of_account_uncovered_outstanding_parent_code,

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

        let config_values = lana_app::credit::ChartOfAccountsIntegrationConfig {
            chart_of_accounts_id: chart.id,
            chart_of_account_facility_omnibus_parent_code:
                chart_of_account_facility_omnibus_parent_code.parse()?,
            chart_of_account_collateral_omnibus_parent_code:
                chart_of_account_collateral_omnibus_parent_code.parse()?,
            chart_of_account_payments_made_omnibus_parent_code:
                chart_of_account_payments_made_omnibus_parent_code.parse()?,
            chart_of_account_interest_added_to_obligations_omnibus_parent_code:
                chart_of_account_interest_added_to_obligations_omnibus_parent_code.parse()?,
            chart_of_account_liquidation_proceeds_omnibus_parent_code:
                chart_of_account_liquidation_proceeds_omnibus_parent_code.parse()?,
            chart_of_account_facility_parent_code: chart_of_account_facility_parent_code.parse()?,
            chart_of_account_collateral_parent_code: chart_of_account_collateral_parent_code
                .parse()?,
            chart_of_account_collateral_in_liquidation_parent_code:
                chart_of_account_collateral_in_liquidation_parent_code.parse()?,
            chart_of_account_interest_income_parent_code:
                chart_of_account_interest_income_parent_code.parse()?,
            chart_of_account_fee_income_parent_code: chart_of_account_fee_income_parent_code
                .parse()?,
            chart_of_account_payment_holding_parent_code: chart_of_account_payment_holding_parent_code
                .parse()?,
            chart_of_account_uncovered_outstanding_parent_code: chart_of_account_uncovered_outstanding_parent_code
                .parse()?,
            chart_of_account_short_term_individual_disbursed_receivable_parent_code:
                chart_of_account_short_term_individual_disbursed_receivable_parent_code.parse()?,
            chart_of_account_short_term_government_entity_disbursed_receivable_parent_code:
                chart_of_account_short_term_government_entity_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_private_company_disbursed_receivable_parent_code:
                chart_of_account_short_term_private_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_bank_disbursed_receivable_parent_code:
                chart_of_account_short_term_bank_disbursed_receivable_parent_code.parse()?,
            chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code:
                chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code:
                chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_individual_disbursed_receivable_parent_code:
                chart_of_account_long_term_individual_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_government_entity_disbursed_receivable_parent_code:
                chart_of_account_long_term_government_entity_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_private_company_disbursed_receivable_parent_code:
                chart_of_account_long_term_private_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_bank_disbursed_receivable_parent_code:
                chart_of_account_long_term_bank_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code:
                chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code:
                chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_individual_interest_receivable_parent_code:
                chart_of_account_short_term_individual_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_government_entity_interest_receivable_parent_code:
                chart_of_account_short_term_government_entity_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_private_company_interest_receivable_parent_code:
                chart_of_account_short_term_private_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_bank_interest_receivable_parent_code:
                chart_of_account_short_term_bank_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
                chart_of_account_short_term_financial_institution_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
                chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_individual_interest_receivable_parent_code:
                chart_of_account_long_term_individual_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_government_entity_interest_receivable_parent_code:
                chart_of_account_long_term_government_entity_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_private_company_interest_receivable_parent_code:
                chart_of_account_long_term_private_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_bank_interest_receivable_parent_code:
                chart_of_account_long_term_bank_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
                chart_of_account_long_term_financial_institution_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
                chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_individual_disbursed_receivable_parent_code:
                chart_of_account_overdue_individual_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_government_entity_disbursed_receivable_parent_code:
                chart_of_account_overdue_government_entity_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_private_company_disbursed_receivable_parent_code:
                chart_of_account_overdue_private_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_bank_disbursed_receivable_parent_code:
                chart_of_account_overdue_bank_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
                chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
                chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code
                    .parse()?
        };

        let config = app
            .credit()
            .chart_of_accounts_integrations()
            .set_config(sub, chart.as_ref(), config_values)
            .await?;
        Ok(CreditModuleConfigurePayload::from(
            CreditModuleConfig::from(config),
        ))
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

    pub async fn pending_credit_facility_collateral_update(
        &self,
        ctx: &Context<'_>,
        input: PendingCreditFacilityCollateralUpdateInput,
    ) -> async_graphql::Result<PendingCreditFacilityCollateralUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let PendingCreditFacilityCollateralUpdateInput {
            pending_credit_facility_id,
            collateral,
            effective,
        } = input;
        exec_mutation!(
            PendingCreditFacilityCollateralUpdatePayload,
            PendingCreditFacility,
            ctx,
            app.credit().update_pending_facility_collateral(
                sub,
                pending_credit_facility_id,
                collateral,
                effective
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

    async fn liquidation_record_collateral_sent(
        &self,
        ctx: &Context<'_>,
        input: LiquidationRecordCollateralSentInput,
    ) -> async_graphql::Result<LiquidationRecordCollateralSentPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            LiquidationRecordCollateralSentPayload,
            Liquidation,
            ctx,
            app.credit().liquidations().record_collateral_sent(
                sub,
                input.liquidation_id.into(),
                input.amount
            )
        )
    }

    async fn liquidation_record_proceeds_received(
        &self,
        ctx: &Context<'_>,
        input: LiquidationRecordProceedsReceivedInput,
    ) -> async_graphql::Result<LiquidationRecordProceedsReceivedPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            LiquidationRecordProceedsReceivedPayload,
            Liquidation,
            ctx,
            app.credit()
                .liquidations()
                .record_proceeds_from_liquidation(sub, input.liquidation_id.into(), input.amount)
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
                .create_custodian(sub, input.name().to_owned(), input.into())
        )
    }

    async fn custodian_config_update(
        &self,
        ctx: &Context<'_>,
        input: CustodianConfigUpdateInput,
    ) -> async_graphql::Result<CustodianConfigUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustodianConfigUpdatePayload,
            Custodian,
            ctx,
            app.custody()
                .update_config(sub, input.custodian_id, input.config.into())
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

    async fn customer_document_download_link_generate(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentDownloadLinksGenerateInput,
    ) -> async_graphql::Result<CustomerDocumentDownloadLinksGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        // not using macro here because DocumentDownloadLinksGeneratePayload is non standard
        let doc = app
            .customers()
            .generate_document_download_link(sub, input.document_id)
            .await?;
        Ok(CustomerDocumentDownloadLinksGeneratePayload::from(doc))
    }

    async fn customer_document_delete(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentDeleteInput,
    ) -> async_graphql::Result<CustomerDocumentDeletePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        // not using macro here because DocumentDeletePayload is non standard
        app.customers()
            .delete_document(sub, input.document_id)
            .await?;
        Ok(CustomerDocumentDeletePayload {
            deleted_document_id: input.document_id,
        })
    }

    async fn customer_document_archive(
        &self,
        ctx: &Context<'_>,
        input: CustomerDocumentArchiveInput,
    ) -> async_graphql::Result<CustomerDocumentArchivePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerDocumentArchivePayload,
            CustomerDocument,
            CustomerDocumentId,
            ctx,
            app.customers().archive_document(sub, input.document_id)
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

    async fn balance_sheet_configure(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<BalanceSheetModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let chart = loader
            .load_one(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {CHART_REF:?}"));

        let config = app
            .accounting()
            .balance_sheets()
            .set_chart_of_accounts_integration_config(
                sub,
                BALANCE_SHEET_NAME.to_string(),
                chart.as_ref(),
            )
            .await?;
        Ok(BalanceSheetModuleConfigurePayload::from(
            BalanceSheetModuleConfig::from(config),
        ))
    }

    async fn profit_and_loss_statement_configure(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<ProfitAndLossStatementModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let chart = loader
            .load_one(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {CHART_REF:?}"));

        let config = app
            .accounting()
            .profit_and_loss()
            .set_chart_of_accounts_integration_config(
                sub,
                PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
                chart.as_ref(),
            )
            .await?;
        Ok(ProfitAndLossStatementModuleConfigurePayload::from(
            ProfitAndLossStatementModuleConfig::from(config),
        ))
    }

    async fn fiscal_year_configure(
        &self,
        ctx: &Context<'_>,
        input: FiscalYearModuleConfigureInput,
    ) -> async_graphql::Result<FiscalYearModuleConfigurePayload> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);

        let FiscalYearModuleConfigureInput { chart_id } = input;

        let config = app.accounting().fiscal_year().configure(chart_id).await?;

        Ok(FiscalYearModuleConfigurePayload::from(
            FiscalYearModuleConfig::from(config),
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

    pub async fn loan_agreement_generate(
        &self,
        ctx: &Context<'_>,
        input: LoanAgreementGenerateInput,
    ) -> async_graphql::Result<LoanAgreementGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        // Create async job for loan agreement generation
        let loan_agreement = app
            .contract_creation()
            .initiate_loan_agreement_generation(sub, input.customer_id)
            .await?;

        let loan_agreement = LoanAgreement::from(loan_agreement);
        Ok(LoanAgreementGeneratePayload::from(loan_agreement))
    }

    async fn loan_agreement_download_link_generate(
        &self,
        ctx: &Context<'_>,
        input: LoanAgreementDownloadLinksGenerateInput,
    ) -> async_graphql::Result<LoanAgreementDownloadLinksGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let doc = app
            .contract_creation()
            .generate_document_download_link(sub, input.loan_agreement_id)
            .await?;
        Ok(LoanAgreementDownloadLinksGeneratePayload::from(doc))
    }

    async fn trigger_report_run(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<ReportRunCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let _job_id = app.reports().trigger_report_run_job(sub).await?;
        Ok(ReportRunCreatePayload { run_id: None })
    }

    async fn report_file_generate_download_link(
        &self,
        ctx: &Context<'_>,
        input: ReportFileGenerateDownloadLinkInput,
    ) -> async_graphql::Result<ReportFileGenerateDownloadLinkPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let url = app
            .reports()
            .generate_report_file_download_link(sub, input.report_id, input.extension)
            .await?;
        Ok(ReportFileGenerateDownloadLinkPayload { url })
    }
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn customer_kyc_updated(
        &self,
        ctx: &Context<'_>,
        customer_id: UUID,
    ) -> async_graphql::Result<impl Stream<Item = CustomerKycUpdatedPayload>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let customer_id = CustomerId::from(customer_id);

        app.customers()
            .find_by_id(sub, customer_id)
            .await?
            .ok_or_else(|| Error::new("Customer not found"))?;

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreCustomerEvent = payload.as_event()?;
            match event {
                CoreCustomerEvent::CustomerAccountKycVerificationUpdated {
                    id,
                    kyc_verification,
                    ..
                } if *id == customer_id => Some(CustomerKycUpdatedPayload {
                    customer_id: *id,
                    kyc_verification: *kyc_verification,
                }),
                _ => None,
            }
        });

        Ok(updates)
    }

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
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::PendingCreditFacilityCollateralizationChanged {
                    id,
                    state,
                    collateral,
                    price,
                    recorded_at,
                    effective,
                } if *id == pending_credit_facility_id => {
                    Some(PendingCreditFacilityCollateralizationPayload {
                        pending_credit_facility_id,
                        update: PendingCreditFacilityCollateralizationUpdated {
                            state: *state,
                            collateral: *collateral,
                            price: price.into_inner(),
                            recorded_at: (*recorded_at).into(),
                            effective: (*effective).into(),
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
                CoreCreditEvent::PendingCreditFacilityCompleted {
                    id,
                    status,
                    recorded_at,
                } if *id == pending_credit_facility_id => {
                    Some(PendingCreditFacilityCompletedPayload {
                        pending_credit_facility_id,
                        update: PendingCreditFacilityCompleted {
                            status: *status,
                            recorded_at: (*recorded_at).into(),
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
                CoreCreditEvent::FacilityProposalConcluded { id, status }
                    if *id == credit_facility_proposal_id =>
                {
                    Some(CreditFacilityProposalConcludedPayload {
                        credit_facility_proposal_id,
                        status: *status,
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
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreCreditEvent = payload.as_event()?;
            match event {
                CoreCreditEvent::FacilityCollateralizationChanged {
                    id,
                    state,
                    recorded_at,
                    effective,
                    collateral,
                    outstanding,
                    price,
                    ..
                } if *id == credit_facility_id => Some(CreditFacilityCollateralizationPayload {
                    credit_facility_id,
                    update: CreditFacilityCollateralizationUpdated {
                        state: *state,
                        collateral: *collateral,
                        outstanding_interest: outstanding.interest,
                        outstanding_disbursal: outstanding.disbursed,
                        recorded_at: (*recorded_at).into(),
                        effective: (*effective).into(),
                        price: price.into_inner(),
                    },
                }),
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

        let stream = app.outbox().listen_persisted(None);
        let updates = stream.filter_map(move |event| async move {
            let payload = event.payload.as_ref()?;
            let event: &CoreAccountingEvent = payload.as_event()?;
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
}
