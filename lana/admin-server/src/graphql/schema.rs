use async_graphql::{types::connection::*, Context, Object};

use lana_app::{
    accounting_init::constants::{
        BALANCE_SHEET_NAME, CASH_FLOW_STATEMENT_NAME, CHART_REF, OBS_CHART_REF,
        OBS_TRIAL_BALANCE_STATEMENT_NAME, PROFIT_AND_LOSS_STATEMENT_NAME,
        TRIAL_BALANCE_STATEMENT_NAME,
    },
    app::LanaApp,
};

use crate::primitives::*;

use super::{
    approval_process::*, audit::*, authenticated_subject::*, chart_of_accounts::*, committee::*,
    credit_facility::*, customer::*, dashboard::*, deposit::*, document::*, financials::*,
    loader::*, new_chart_of_accounts::*, policy::*, price::*, report::*, sumsub::*,
    terms_template::*, user::*, withdrawal::*,
};

pub struct Query;

#[Object]
impl Query {
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<AuthenticatedSubject> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let user = Arc::new(app.users().find_for_subject(sub).await?);
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
        maybe_fetch_one!(User, ctx, app.users().find_by_id(sub, id))
    }

    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let users: Vec<_> = app
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
        let sort = sort.unwrap_or_default();
        list_with_combo_cursor!(
            CustomersCursor,
            Customer,
            DomainCustomersSortBy::from(sort),
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
            app.terms_templates().find_by_id(sub, id)
        )
    }

    async fn terms_templates(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<TermsTemplate>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let terms_templates = app.terms_templates().list(sub).await?;
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
            app.credit_facilities().find_by_id(sub, id)
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
            |query| app.credit_facilities().list(sub, query, filter, sort)
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
            app.credit_facilities().find_disbursal_by_id(sub, id)
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
            |query| {
                app.credit_facilities()
                    .list_disbursals(sub, query, filter, sort)
            }
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

    async fn trial_balance(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<TrialBalance> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let account_summary = app
            .trial_balances()
            .trial_balance(
                sub,
                TRIAL_BALANCE_STATEMENT_NAME.to_string(),
                from.into_inner(),
                until.map(|t| t.into_inner()),
            )
            .await?;
        Ok(TrialBalance::from(account_summary))
    }

    async fn off_balance_sheet_trial_balance(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<TrialBalance> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let account_summary = app
            .trial_balances()
            .trial_balance(
                sub,
                OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string(),
                from.into_inner(),
                until.map(|t| t.into_inner()),
            )
            .await?;
        Ok(TrialBalance::from(account_summary))
    }

    async fn chart_of_accounts(&self, ctx: &Context<'_>) -> async_graphql::Result<ChartOfAccounts> {
        let reference = CHART_REF.to_string();

        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let chart = app
            .chart_of_accounts()
            .list_charts(sub)
            .await?
            .into_iter()
            .find(|p| p.reference == reference)
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {}", reference));
        Ok(ChartOfAccounts::from(chart))
    }

    async fn new_chart_of_accounts(
        &self,
        ctx: &Context<'_>,
        reference: String, // TODO: remove and use CHART_REF
    ) -> async_graphql::Result<NewChartOfAccounts> {
        // let reference = CHART_REF.to_string();

        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let chart = app
            .new_chart_of_accounts()
            .find_by_reference(sub, reference.to_string())
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {}", reference));
        Ok(NewChartOfAccounts::from(chart))
    }

    async fn off_balance_sheet_chart_of_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<ChartOfAccounts> {
        let reference = OBS_CHART_REF.to_string();

        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let chart = app
            .chart_of_accounts()
            .list_charts(sub)
            .await?
            .into_iter()
            .find(|p| p.reference == reference)
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {}", reference));
        Ok(ChartOfAccounts::from(chart))
    }

    async fn balance_sheet(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<BalanceSheet> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let balance_sheet = app
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
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<ProfitAndLossStatement> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let profit_and_loss = app
            .profit_and_loss_statements()
            .pl_statement(
                sub,
                PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
                from.into_inner(),
                until.map(|t| t.into_inner()),
            )
            .await?;
        Ok(ProfitAndLossStatement::from(profit_and_loss))
    }

    async fn cash_flow_statement(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<CashFlowStatement> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let cash_flow = app
            .cash_flow_statements()
            .cash_flow_statement(
                sub,
                CASH_FLOW_STATEMENT_NAME.to_string(),
                from.into_inner(),
                until.map(|t| t.into_inner()),
            )
            .await?;
        Ok(CashFlowStatement::from(cash_flow))
    }

    #[allow(unused_variables)]
    async fn account_set(
        &self,
        ctx: &Context<'_>,
        account_set_id: UUID,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<Option<AccountSetAndSubAccounts>> {
        unimplemented!()
        // let (app, sub) = app_and_sub_from_ctx!(ctx);
        // let account_set = app
        //     .ledger()
        //     .account_set_and_sub_accounts_with_balance(
        //         sub,
        //         uuid::Uuid::from(&account_set_id).into(),
        //         0,
        //         None,
        //         from.into_inner(),
        //         until.map(|t| t.into_inner()),
        //     )
        //     .await?;
        // Ok(account_set.map(|a| {
        // AccountSetAndSubAccounts::from((from.into_inner(), until.map(|t| t.into_inner()), a))
        // }))
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
        let file = input.file.value(ctx)?;
        exec_mutation!(
            DocumentCreatePayload,
            Document,
            ctx,
            app.documents()
                .create(sub, file.content.to_vec(), input.customer_id, file.filename)
        )
    }

    pub async fn sumsub_permalink_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubPermalinkCreateInput,
    ) -> async_graphql::Result<SumsubPermalinkCreatePayload> {
        let app = ctx.data_unchecked::<LanaApp>();
        let res = app.applicants().create_permalink(input.customer_id).await?;

        let url = res.url;
        Ok(SumsubPermalinkCreatePayload { url })
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
            app.users().create_user(sub, input.email)
        )
    }

    async fn user_assign_role(
        &self,
        ctx: &Context<'_>,
        input: UserAssignRoleInput,
    ) -> async_graphql::Result<UserAssignRolePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let UserAssignRoleInput { id, role } = input;
        exec_mutation!(
            UserAssignRolePayload,
            User,
            ctx,
            app.users().assign_role_to_user(sub, id, role)
        )
    }

    async fn user_revoke_role(
        &self,
        ctx: &Context<'_>,
        input: UserRevokeRoleInput,
    ) -> async_graphql::Result<UserRevokeRolePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let UserRevokeRoleInput { id, role } = input;
        exec_mutation!(
            UserRevokeRolePayload,
            User,
            ctx,
            app.users().revoke_role_from_user(sub, id, role)
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
            app.customers().create(sub, input.email, input.telegram_id)
        )
    }

    async fn customer_update(
        &self,
        ctx: &Context<'_>,
        input: CustomerUpdateInput,
    ) -> async_graphql::Result<CustomerUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CustomerUpdatePayload,
            Customer,
            ctx,
            app.customers()
                .update(sub, input.customer_id, input.telegram_id)
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
            .incurrence_interval(input.incurrence_interval)
            .one_time_fee_rate(input.one_time_fee_rate)
            .duration(input.duration)
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;

        exec_mutation!(
            TermsTemplateCreatePayload,
            TermsTemplate,
            ctx,
            app.terms_templates()
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
            .incurrence_interval(input.incurrence_interval)
            .one_time_fee_rate(input.one_time_fee_rate)
            .duration(input.duration)
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;
        exec_mutation!(
            TermsTemplateUpdatePayload,
            TermsTemplate,
            ctx,
            app.terms_templates().update_term_values(
                sub,
                TermsTemplateId::from(input.id),
                term_values
            )
        )
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
            .incurrence_interval(terms.incurrence_interval)
            .one_time_fee_rate(terms.one_time_fee_rate)
            .duration(terms.duration)
            .liquidation_cvl(terms.liquidation_cvl)
            .margin_call_cvl(terms.margin_call_cvl)
            .initial_cvl(terms.initial_cvl)
            .build()?;

        exec_mutation!(
            CreditFacilityCreatePayload,
            CreditFacility,
            ctx,
            app.credit_facilities().initiate(
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
        } = input;
        exec_mutation!(
            CreditFacilityCollateralUpdatePayload,
            CreditFacility,
            ctx,
            app.credit_facilities()
                .update_collateral(sub, credit_facility_id.into(), collateral)
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
            app.credit_facilities().record_payment(
                sub,
                input.credit_facility_id.into(),
                input.amount
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
            app.credit_facilities().initiate_disbursal(
                sub,
                input.credit_facility_id.into(),
                input.amount
            )
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
            app.credit_facilities()
                .complete_facility(sub, input.credit_facility_id)
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

    #[allow(unused_variables)]
    pub async fn shareholder_equity_add(
        &self,
        ctx: &Context<'_>,
        input: ShareholderEquityAddInput,
    ) -> async_graphql::Result<SuccessPayload> {
        unimplemented!()
        // let app = ctx.data_unchecked::<LanaApp>();
        // Ok(SuccessPayload::from(
        //     app.ledger()
        //         .add_equity(input.amount, input.reference)
        //         .await?,
        // ))
    }
}
