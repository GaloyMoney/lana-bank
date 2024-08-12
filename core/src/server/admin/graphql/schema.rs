use async_graphql::{types::connection::*, Context, Object};
use uuid::Uuid;

use super::{
    account_set::*, audit::AuditEntry, customer::*, deposit::*, loan::*, shareholder_equity::*,
    terms::*, user::*, withdraw::*,
};
use crate::{
    app::LavaApp,
    audit::AuditCursor,
    primitives::{CustomerId, LoanId, UserId},
    server::{
        admin::AdminAuthContext,
        shared_graphql::{
            customer::Customer, deposit::Deposit, loan::Loan, objects::SuccessPayload,
            primitives::UUID, sumsub::SumsubPermalinkCreatePayload, terms::Terms,
            withdraw::Withdrawal,
        },
    },
};

pub struct Query;

#[Object]
impl Query {
    async fn audit(
        &self,
        ctx: &Context<'_>,
        first: i64,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<AuditCursor, AuditEntry>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let after_cursor = after.map(|cursor| AuditCursor {
            id: cursor.parse::<i64>().unwrap(),
        });

        let query_args = crate::query::PaginatedQueryArgs {
            first: first.try_into().expect("convert to usize failed"),
            after: after_cursor,
        };

        let res = app.list_audit(sub, query_args).await?;

        let mut connection = Connection::new(false, res.has_next_page);
        connection
            .edges
            .extend(res.entities.into_iter().map(|entry| {
                let cursor = AuditCursor { id: entry.id.0 };
                Edge::new(cursor, AuditEntry::from(entry))
            }));

        Ok(connection)
    }

    async fn loan(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Loan>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let loan = app.loans().find_by_id(Some(sub), LoanId::from(id)).await?;
        Ok(loan.map(Loan::from))
    }

    async fn customer(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Customer>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let customer = app.customers().find_by_id(Some(sub), id).await?;
        Ok(customer.map(Customer::from))
    }

    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let user = app.users().find_for_subject(sub).await?;
        Ok(User::from(user))
    }

    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let users = app.users().list_users(sub).await?;
        Ok(users.into_iter().map(User::from).collect())
    }

    async fn user(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<User>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let user = app.users().find_by_id(sub, UserId::from(id)).await?;
        Ok(user.map(User::from))
    }

    async fn customers(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<CustomerByNameCursor, Customer, EmptyFields, EmptyFields>>
    {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .customers()
                    .list(
                        sub,
                        crate::query::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::customer::CustomerByNameCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|user| {
                        let cursor = CustomerByNameCursor::from((user.id, user.email.as_ref()));
                        Edge::new(cursor, Customer::from(user))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn loans(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<LoanCursor, Loan, EmptyFields, EmptyFields>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .loans()
                    .list(
                        sub,
                        crate::query::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::loan::LoanCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|loan| {
                        let cursor = LoanCursor::from(loan.id);
                        Edge::new(cursor, Loan::from(loan))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn default_terms(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Terms>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let terms = app.loans().find_default_terms(sub).await?;
        Ok(terms.map(Terms::from))
    }

    async fn trial_balance(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<TrialBalance>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let account_summary = app.ledger().trial_balance(sub).await?;
        Ok(account_summary.map(TrialBalance::from))
    }

    async fn off_balance_sheet_trial_balance(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<TrialBalance>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let account_summary = app.ledger().obs_trial_balance(sub).await?;
        Ok(account_summary.map(TrialBalance::from))
    }

    async fn chart_of_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ChartOfAccounts>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let chart_of_accounts = app.ledger().chart_of_accounts(sub).await?;
        Ok(chart_of_accounts.map(ChartOfAccounts::from))
    }

    async fn off_balance_sheet_chart_of_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ChartOfAccounts>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let chart_of_accounts = app.ledger().obs_chart_of_accounts(sub).await?;
        Ok(chart_of_accounts.map(ChartOfAccounts::from))
    }

    async fn balance_sheet(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<BalanceSheet>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let balance_sheet = app.ledger().balance_sheet(sub).await?;
        Ok(balance_sheet.map(BalanceSheet::from))
    }

    async fn profit_and_loss_statement(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ProfitAndLossStatement>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let profit_and_loss = app.ledger().profit_and_loss(sub).await?;
        Ok(profit_and_loss.map(ProfitAndLossStatement::from))
    }

    async fn account_set_with_balance(
        &self,
        ctx: &Context<'_>,
        account_set_id: UUID,
    ) -> async_graphql::Result<Option<AccountSetAndSubAccountsWithBalance>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let account_set = app
            .ledger()
            .account_set_and_sub_accounts_with_balance(sub, account_set_id.into(), 0, None)
            .await?;
        Ok(account_set.map(AccountSetAndSubAccountsWithBalance::from))
    }

    async fn deposit(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Deposit>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let deposit = app.deposits().find_by_id(sub, id).await?;
        Ok(deposit.map(Deposit::from))
    }

    async fn deposits(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<DepositCursor, Deposit, EmptyFields, EmptyFields>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .deposits()
                    .list(
                        sub,
                        crate::query::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::deposit::DepositCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|deposit| {
                        let cursor = DepositCursor::from(deposit.created_at());
                        Edge::new(cursor, Deposit::from(deposit))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn withdrawal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Withdrawal>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let deposit = app.withdraws().find_by_id(sub, id).await?;
        Ok(deposit.map(Withdrawal::from))
    }

    async fn withdrawals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<WithdrawCursor, Withdrawal, EmptyFields, EmptyFields>>
    {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .withdraws()
                    .list(
                        sub,
                        crate::query::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::withdraw::WithdrawCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|withdraw| {
                        let cursor = WithdrawCursor::from(withdraw.created_at());
                        Edge::new(cursor, Withdrawal::from(withdraw))
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
    pub async fn shareholder_equity_add(
        &self,
        ctx: &Context<'_>,
        input: ShareholderEquityAddInput,
    ) -> async_graphql::Result<SuccessPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        Ok(SuccessPayload::from(
            app.ledger()
                .add_equity(input.amount, input.reference)
                .await?,
        ))
    }

    pub async fn sumsub_permalink_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubPermalinkCreateInput,
    ) -> async_graphql::Result<SumsubPermalinkCreatePayload> {
        let customer_id = Uuid::parse_str(&input.customer_id);
        let customer_id = customer_id.map_err(|_| "Invalid user id")?;
        let customer_id = CustomerId::from(customer_id);

        let app = ctx.data_unchecked::<LavaApp>();
        let res = app.applicants().create_permalink(customer_id).await?;

        let url = res.url;
        Ok(SumsubPermalinkCreatePayload { url })
    }

    async fn default_terms_update(
        &self,
        ctx: &Context<'_>,
        input: DefaultTermsUpdateInput,
    ) -> async_graphql::Result<DefaultTermsUpdatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let term_values = crate::loan::TermValues::builder()
            .annual_rate(input.annual_rate)
            .interval(input.interval)
            .duration(input.duration)
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;
        let terms = app.loans().update_default_terms(sub, term_values).await?;
        Ok(DefaultTermsUpdatePayload::from(terms))
    }

    async fn loan_create(
        &self,
        ctx: &Context<'_>,
        input: LoanCreateInput,
    ) -> async_graphql::Result<LoanCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let LoanCreateInput {
            customer_id,
            desired_principal,
            loan_terms,
        } = input;
        let term_values = crate::loan::TermValues::builder()
            .annual_rate(loan_terms.annual_rate)
            .interval(loan_terms.interval)
            .duration(loan_terms.duration)
            .liquidation_cvl(loan_terms.liquidation_cvl)
            .margin_call_cvl(loan_terms.margin_call_cvl)
            .initial_cvl(loan_terms.initial_cvl)
            .build()?;
        let loan = app
            .loans()
            .create_loan_for_customer(sub, customer_id, desired_principal, term_values)
            .await?;
        Ok(LoanCreatePayload::from(loan))
    }

    async fn loan_approve(
        &self,
        ctx: &Context<'_>,
        input: LoanApproveInput,
    ) -> async_graphql::Result<LoanApprovePayload> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let loan = app
            .loans()
            .approve_loan(sub, input.loan_id, input.collateral)
            .await?;
        Ok(LoanApprovePayload::from(loan))
    }

    pub async fn loan_partial_payment(
        &self,
        ctx: &Context<'_>,
        input: LoanPartialPaymentInput,
    ) -> async_graphql::Result<LoanPartialPaymentPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let loan = app
            .loans()
            .record_payment(sub, input.loan_id.into(), input.amount)
            .await?;
        Ok(LoanPartialPaymentPayload::from(loan))
    }

    pub async fn deposit_record(
        &self,
        ctx: &Context<'_>,
        input: DepositRecordInput,
    ) -> async_graphql::Result<DepositRecordPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let deposit = app
            .deposits()
            .record(sub, input.customer_id, input.amount, input.reference)
            .await?;

        Ok(DepositRecordPayload::from(deposit))
    }

    pub async fn withdrawal_initiate(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalInitiateInput,
    ) -> async_graphql::Result<WithdrawalInitiatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let withdraw = app
            .withdraws()
            .initiate(sub, input.customer_id, input.amount, input.reference)
            .await?;

        Ok(WithdrawalInitiatePayload::from(withdraw))
    }

    pub async fn withdrawal_confirm(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalConfirmInput,
    ) -> async_graphql::Result<WithdrawalConfirmPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let withdraw = app.withdraws().confirm(sub, input.withdrawal_id).await?;

        Ok(WithdrawalConfirmPayload::from(withdraw))
    }

    async fn customer_create(
        &self,
        ctx: &Context<'_>,
        input: CustomerCreateInput,
    ) -> async_graphql::Result<CustomerCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let customer = app
            .customers()
            .create_customer_through_admin(sub, input.email)
            .await?;
        Ok(CustomerCreatePayload::from(customer))
    }

    async fn user_create(
        &self,
        ctx: &Context<'_>,
        input: UserCreateInput,
    ) -> async_graphql::Result<UserCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let user = app.users().create_user(sub, input.email).await?;
        Ok(UserCreatePayload::from(user))
    }

    async fn user_assign_role(
        &self,
        ctx: &Context<'_>,
        input: UserAssignRoleInput,
    ) -> async_graphql::Result<UserAssignRolePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let UserAssignRoleInput { id, role } = input;
        let user = app
            .users()
            .assign_role_to_user(sub, id.into(), role)
            .await?;
        Ok(UserAssignRolePayload::from(user))
    }

    async fn user_revoke_role(
        &self,
        ctx: &Context<'_>,
        input: UserRevokeRoleInput,
    ) -> async_graphql::Result<UserRevokeRolePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let UserRevokeRoleInput { id, role } = input;
        let user = app
            .users()
            .revoke_role_from_user(sub, id.into(), role)
            .await?;
        Ok(UserRevokeRolePayload::from(user))
    }
}
