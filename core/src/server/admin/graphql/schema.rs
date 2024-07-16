use async_graphql::{types::connection::*, *};
use uuid::Uuid;

use super::{account_set::*, loan::*, shareholder_equity::*, terms::*, user::*};
use crate::{
    app::LavaApp,
    primitives::{LoanId, UserId},
    server::shared_graphql::{
        loan::Loan, objects::SuccessPayload, primitives::UUID,
        sumsub::SumsubPermalinkCreatePayload, terms::Terms, user::User,
    },
};

pub struct Query;

#[Object]
impl Query {
    async fn loan(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Loan>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let loan = app.loans().find_by_id(LoanId::from(id)).await?;
        Ok(loan.map(Loan::from))
    }

    async fn user(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<User>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app.users().find_by_id(UserId::from(id)).await?;
        Ok(user.map(User::from))
    }

    async fn users(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> Result<Connection<UserByNameCursor, User, EmptyFields, EmptyFields>> {
        let app = ctx.data_unchecked::<LavaApp>();
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .users()
                    .list(crate::query::PaginatedQueryArgs {
                        first,
                        after: after.map(crate::user::UserByNameCursor::from),
                    })
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|user| {
                        let cursor = UserByNameCursor::from((user.id, user.email.as_ref()));
                        Edge::new(cursor, User::from(user))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn trial_balance(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AccountSetAndMemberBalances>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let account_summary = app.ledger().trial_balance().await?;
        Ok(account_summary.map(AccountSetAndMemberBalances::from))
    }

    async fn off_balance_sheet_trial_balance(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<AccountSetAndMemberBalances>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let account_summary = app.ledger().obs_trial_balance().await?;
        Ok(account_summary.map(AccountSetAndMemberBalances::from))
    }

    async fn chart_of_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ChartOfAccounts>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let chart_of_accounts = app.ledger().chart_of_accounts().await?;
        Ok(chart_of_accounts.map(ChartOfAccounts::from))
    }

    async fn off_balance_sheet_chart_of_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ChartOfAccounts>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let chart_of_accounts = app.ledger().obs_chart_of_accounts().await?;
        Ok(chart_of_accounts.map(ChartOfAccounts::from))
    }

    async fn chart_of_accounts_account_set(
        &self,
        ctx: &Context<'_>,
        account_set_id: UUID,
    ) -> async_graphql::Result<Option<ChartOfAccountsAccountSet>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let chart_of_accounts = app
            .ledger()
            .chart_of_accounts_account_set(account_set_id.into(), 0, None)
            .await?;
        Ok(chart_of_accounts.map(ChartOfAccountsAccountSet::from))
    }

    async fn current_terms(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Terms>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let current_terms = app.loans().find_current_terms().await?;
        Ok(current_terms.map(Terms::from))
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
        let user_id = Uuid::parse_str(&input.user_id);
        let user_id = user_id.map_err(|_| "Invalid user id")?;
        let user_id = UserId::from(user_id);

        let app = ctx.data_unchecked::<LavaApp>();
        let res = app.applicants().create_permalink(user_id).await?;

        let url = res.url;
        Ok(SumsubPermalinkCreatePayload { url })
    }

    async fn current_terms_update(
        &self,
        ctx: &Context<'_>,
        input: CurrentTermsUpdateInput,
    ) -> async_graphql::Result<CurrentTermsUpdatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let term_values = crate::loan::TermValues::builder()
            .annual_rate(input.annual_rate)
            .interval(input.interval)
            .duration(input.duration)
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;
        let terms = app.loans().update_current_terms(term_values).await?;
        Ok(CurrentTermsUpdatePayload::from(terms))
    }

    async fn loan_create(
        &self,
        ctx: &Context<'_>,
        input: LoanCreateInput,
    ) -> async_graphql::Result<LoanCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let loan = app
            .loans()
            .create_loan_for_user(input.user_id, input.desired_principal)
            .await?;
        Ok(LoanCreatePayload::from(loan))
    }

    async fn loan_approve(
        &self,
        ctx: &Context<'_>,
        input: LoanApproveInput,
    ) -> async_graphql::Result<LoanApprovePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let loan = app
            .loans()
            .approve_loan(input.loan_id, input.collateral)
            .await?;
        Ok(LoanApprovePayload::from(loan))
    }

    pub async fn loan_partial_payment(
        &self,
        ctx: &Context<'_>,
        input: LoanPartialPaymentInput,
    ) -> async_graphql::Result<LoanPartialPaymentPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let loan = app
            .loans()
            .record_payment(input.loan_id.into(), input.amount)
            .await?;
        Ok(LoanPartialPaymentPayload::from(loan))
    }
}
