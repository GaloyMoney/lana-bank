use async_graphql::{types::connection::*, Context, Object};

use super::{
    account_set::*,
    audit::{AuditCursor, AuditEntry},
    credit_facility::*,
    customer::*,
    deposit::*,
    loan::*,
    price::*,
    report::*,
    shareholder_equity::*,
    terms_template::TermsTemplate,
    user::*,
    withdraw::*,
};

use crate::{
    app::LavaApp,
    credit_facility::CreditFacilityByCreatedAtCursor,
    primitives::{
        CreditFacilityId, CustomerId, DocumentId, LoanId, ReportId, TermsTemplateId, UserId,
    },
    server::{
        admin::{
            graphql::terms_template::{
                TermsTemplateCreateInput, TermsTemplateCreatePayload, TermsTemplateUpdateInput,
                TermsTemplateUpdatePayload,
            },
            AdminAuthContext,
        },
        shared_graphql::{
            customer::Customer,
            deposit::Deposit,
            document::{
                Document, DocumentArchiveInput, DocumentArchivePayload, DocumentCreateInput,
                DocumentCreatePayload, DocumentDeleteInput, DocumentDeletePayload,
                DocumentDownloadLinksGenerateInput, DocumentDownloadLinksGeneratePayload,
            },
            loan::Loan,
            objects::SuccessPayload,
            primitives::{Timestamp, UUID},
            sumsub::SumsubPermalinkCreatePayload,
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
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<AuditCursor, AuditEntry>> {
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
                    .list_audit(
                        sub,
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::audit::AuditCursor::from),
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

    async fn loan(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Loan>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let loan = app.loans().find_by_id(Some(sub), LoanId::from(id)).await?;
        Ok(loan.map(Loan::from))
    }

    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacility>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let credit_facility = app
            .credit_facilities()
            .find_by_id(Some(sub), CreditFacilityId::from(id))
            .await?;
        Ok(credit_facility.map(CreditFacility::from))
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

    async fn customer_by_email(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> async_graphql::Result<Option<Customer>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let customer = app.customers().find_by_email(sub, email).await?;
        Ok(customer.map(Customer::from))
    }

    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let app = ctx.data_unchecked::<LavaApp>();
        let auth_ctx: &AdminAuthContext = ctx.data()?;

        let my_id = auth_ctx.authenticated_user_id();
        let user = app.users().find_by_id_internal(my_id).await?;
        let user = user.expect("User always exists");
        Ok(User::from(user))
    }

    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let users = app.users().list_users(sub).await?;
        Ok(users.into_iter().map(User::from).collect())
    }

    async fn terms_templates(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<TermsTemplate>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let terms_templates = app.terms_templates().list(sub).await?;
        Ok(terms_templates
            .into_iter()
            .map(TermsTemplate::from)
            .collect())
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
    ) -> async_graphql::Result<Connection<CustomerByEmailCursor, Customer, EmptyFields, EmptyFields>>
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
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::customer::CustomerByEmailCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|user| {
                        let cursor = CustomerByEmailCursor::from((user.id, user.email.as_ref()));
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
    ) -> async_graphql::Result<
        Connection<LoanByCollateralizationRatioCursor, Loan, EmptyFields, EmptyFields>,
    > {
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
                    .list_by_collateralization_ratio(
                        sub,
                        es_entity::PaginatedQueryArgs { first, after },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|loan| {
                        let cursor = LoanByCollateralizationRatioCursor::from(&loan);
                        Edge::new(cursor, Loan::from(loan))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<CreditFacilityByCreatedAtCursor, CreditFacility, EmptyFields, EmptyFields>,
    > {
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
                    .credit_facilities()
                    .list(sub, es_entity::PaginatedQueryArgs { first, after })
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|credit_facility| {
                        let cursor = CreditFacilityByCreatedAtCursor::from(&credit_facility);
                        Edge::new(cursor, CreditFacility::from(credit_facility))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn trial_balance(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<Option<TrialBalance>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let account_summary = app
            .ledger()
            .trial_balance(sub, from.into_inner(), until.map(|t| t.into_inner()))
            .await?;
        Ok(account_summary.map(TrialBalance::from))
    }

    async fn off_balance_sheet_trial_balance(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<Option<TrialBalance>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let account_summary = app
            .ledger()
            .obs_trial_balance(sub, from.into_inner(), until.map(|t| t.into_inner()))
            .await?;
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
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<Option<BalanceSheet>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let balance_sheet = app
            .ledger()
            .balance_sheet(sub, from.into_inner(), until.map(|t| t.into_inner()))
            .await?;
        Ok(balance_sheet.map(BalanceSheet::from))
    }

    async fn profit_and_loss_statement(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<Option<ProfitAndLossStatement>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let profit_and_loss = app
            .ledger()
            .profit_and_loss(sub, from.into_inner(), until.map(|t| t.into_inner()))
            .await?;
        Ok(profit_and_loss.map(ProfitAndLossStatement::from))
    }

    async fn cash_flow_statement(
        &self,
        ctx: &Context<'_>,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<Option<CashFlowStatement>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let cash_flow = app
            .ledger()
            .cash_flow(sub, from.into_inner(), until.map(|t| t.into_inner()))
            .await?;
        Ok(cash_flow.map(CashFlowStatement::from))
    }

    async fn account_set(
        &self,
        ctx: &Context<'_>,
        account_set_id: UUID,
        from: Timestamp,
        until: Option<Timestamp>,
    ) -> async_graphql::Result<Option<AccountSetAndSubAccounts>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let account_set = app
            .ledger()
            .account_set_and_sub_accounts_with_balance(
                sub,
                account_set_id.into(),
                0,
                None,
                from.clone().into_inner(),
                until.clone().map(|t| t.into_inner()),
            )
            .await?;
        Ok(account_set.map(|a| {
            AccountSetAndSubAccounts::from((from.into_inner(), until.map(|t| t.into_inner()), a))
        }))
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
    ) -> async_graphql::Result<
        Connection<DepositByCreatedAtCursor, Deposit, EmptyFields, EmptyFields>,
    > {
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
                    .list(sub, es_entity::PaginatedQueryArgs { first, after })
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|deposit| {
                        let cursor = DepositByCreatedAtCursor::from(&deposit);
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
    ) -> async_graphql::Result<Connection<WithdrawByIdCursor, Withdrawal, EmptyFields, EmptyFields>>
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
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after.map(crate::withdraw::WithdrawByIdCursor::from),
                        },
                    )
                    .await?;
                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|withdraw| {
                        let cursor = WithdrawByIdCursor::from(withdraw.id);
                        Edge::new(cursor, Withdrawal::from(withdraw))
                    }));
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn realtime_price(&self, ctx: &Context<'_>) -> async_graphql::Result<RealtimePrice> {
        let app = ctx.data_unchecked::<LavaApp>();
        let usd_cents_per_btc = app.price().usd_cents_per_btc().await?;
        Ok(usd_cents_per_btc.into())
    }

    async fn reports(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Report>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let users = app.reports().list_reports(sub).await?;
        Ok(users.into_iter().map(Report::from).collect())
    }

    async fn report(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Report>> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let report = app.reports().find_by_id(sub, ReportId::from(id)).await?;
        Ok(report.map(Report::from))
    }

    async fn terms_template(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<TermsTemplate>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let terms_template = app
            .terms_templates()
            .find_by_id(sub, TermsTemplateId::from(id))
            .await?;
        Ok(terms_template.map(TermsTemplate::from))
    }

    async fn document(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<Document>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let document = app
            .documents()
            .find_by_id(sub, DocumentId::from(id))
            .await?;
        Ok(document.map(Document::from))
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
        let app = ctx.data_unchecked::<LavaApp>();
        let file = input.file.value(ctx)?;
        let AdminAuthContext { sub } = ctx.data()?;

        let document = app
            .documents()
            .create(sub, file.content.to_vec(), input.customer_id, file.filename)
            .await?;

        Ok(DocumentCreatePayload::from(document))
    }

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
        let app = ctx.data_unchecked::<LavaApp>();
        let res = app.applicants().create_permalink(input.customer_id).await?;

        let url = res.url;
        Ok(SumsubPermalinkCreatePayload { url })
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
        let term_values = crate::terms::TermValues::builder()
            .annual_rate(loan_terms.annual_rate)
            .accrual_interval(loan_terms.accrual_interval)
            .incurrence_interval(loan_terms.incurrence_interval)
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

        let loan = app.loans().add_approval(sub, input.loan_id).await?;
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
            .record_payment_or_complete_loan(sub, input.loan_id.into(), input.amount)
            .await?;
        Ok(LoanPartialPaymentPayload::from(loan))
    }

    pub async fn loan_collateral_update(
        &self,
        ctx: &Context<'_>,
        input: LoanCollateralUpdateInput,
    ) -> async_graphql::Result<LoanCollateralUpdatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let LoanCollateralUpdateInput {
            loan_id,
            collateral,
        } = input;
        let loan = app
            .loans()
            .update_collateral(sub, loan_id.into(), collateral)
            .await?;
        Ok(LoanCollateralUpdatePayload::from(loan))
    }

    pub async fn collateralization_state_update(
        &self,
        ctx: &Context<'_>,
        input: CollateralizationStateUpdateInput,
    ) -> async_graphql::Result<CollateralizationStateUpdatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let CollateralizationStateUpdateInput { loan_id } = input;
        let loan = app
            .loans()
            .update_collateralization_state(sub, loan_id.into())
            .await?;
        Ok(CollateralizationStateUpdatePayload::from(loan))
    }

    pub async fn credit_facility_create(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCreateInput,
    ) -> async_graphql::Result<CreditFacilityCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let CreditFacilityCreateInput {
            facility,
            customer_id,
            terms,
        } = input;

        let credit_facility_term_values = crate::terms::TermValues::builder()
            .annual_rate(terms.annual_rate)
            .accrual_interval(terms.accrual_interval)
            .incurrence_interval(terms.incurrence_interval)
            .duration(terms.duration)
            .liquidation_cvl(terms.liquidation_cvl)
            .margin_call_cvl(terms.margin_call_cvl)
            .initial_cvl(terms.initial_cvl)
            .build()?;

        let credit_facility = app
            .credit_facilities()
            .create(sub, customer_id, facility, credit_facility_term_values)
            .await?;

        Ok(CreditFacilityCreatePayload::from(credit_facility))
    }

    async fn credit_facility_approve(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityApproveInput,
    ) -> async_graphql::Result<CreditFacilityApprovePayload> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let credit_facility = app
            .credit_facilities()
            .add_approval(sub, input.credit_facility_id)
            .await?;
        Ok(CreditFacilityApprovePayload::from(credit_facility))
    }

    async fn credit_facility_complete(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCompleteInput,
    ) -> async_graphql::Result<CreditFacilityCompletePayload> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let credit_facility = app
            .credit_facilities()
            .complete_facility(sub, input.credit_facility_id)
            .await?;

        Ok(CreditFacilityCompletePayload::from(credit_facility))
    }

    pub async fn credit_facility_collateral_update(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCollateralUpdateInput,
    ) -> async_graphql::Result<CreditFacilityCollateralUpdatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let CreditFacilityCollateralUpdateInput {
            credit_facility_id,
            collateral,
        } = input;
        let credit_facility = app
            .credit_facilities()
            .update_collateral(sub, credit_facility_id.into(), collateral)
            .await?;
        Ok(CreditFacilityCollateralUpdatePayload::from(credit_facility))
    }

    pub async fn credit_facility_partial_payment(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let credit_facility = app
            .credit_facilities()
            .record_payment(sub, input.credit_facility_id.into(), input.amount)
            .await?;
        Ok(CreditFacilityPartialPaymentPayload::from(credit_facility))
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

    pub async fn withdrawal_cancel(
        &self,
        ctx: &Context<'_>,
        input: WithdrawalCancelInput,
    ) -> async_graphql::Result<WithdrawalCancelPayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let withdraw = app.withdraws().cancel(sub, input.withdrawal_id).await?;

        Ok(WithdrawalCancelPayload::from(withdraw))
    }

    pub async fn credit_facility_disbursement_initiate(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityDisbursementInitiateInput,
    ) -> async_graphql::Result<CreditFacilityDisbursementInitiatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let disbursement = app
            .credit_facilities()
            .initiate_disbursement(sub, input.credit_facility_id.into(), input.amount)
            .await?;

        Ok(CreditFacilityDisbursementInitiatePayload::from(
            disbursement,
        ))
    }

    async fn credit_facility_disbursement_approve(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityDisbursementApproveInput,
    ) -> async_graphql::Result<CreditFacilityDisbursementApprovePayload> {
        let app = ctx.data_unchecked::<LavaApp>();

        let AdminAuthContext { sub } = ctx.data()?;

        let credit_facility = app
            .credit_facilities()
            .add_disbursement_approval(sub, input.credit_facility_id.into(), input.disbursement_idx)
            .await?;
        Ok(CreditFacilityDisbursementApprovePayload::from(
            credit_facility,
        ))
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
            .create_customer_through_admin(sub, input.email, input.telegram_id)
            .await?;
        Ok(CustomerCreatePayload::from(customer))
    }

    async fn customer_update(
        &self,
        ctx: &Context<'_>,
        input: CustomerUpdateInput,
    ) -> async_graphql::Result<CustomerUpdatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let customer = app
            .customers()
            .update(sub, CustomerId::from(input.customer_id), input.telegram_id)
            .await?;
        Ok(CustomerUpdatePayload::from(customer))
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

    async fn report_create(&self, ctx: &Context<'_>) -> async_graphql::Result<ReportCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let report = app.reports().create(sub).await?;
        Ok(ReportCreatePayload::from(report))
    }

    async fn report_download_links_generate(
        &self,
        ctx: &Context<'_>,
        input: ReportDownloadLinksGenerateInput,
    ) -> async_graphql::Result<ReportDownloadLinksGeneratePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let links = app
            .reports()
            .generate_download_links(sub, input.report_id.into())
            .await?;
        Ok(ReportDownloadLinksGeneratePayload::from(links))
    }

    async fn document_download_link_generate(
        &self,
        ctx: &Context<'_>,
        input: DocumentDownloadLinksGenerateInput,
    ) -> async_graphql::Result<DocumentDownloadLinksGeneratePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
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
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let document_id = input.document_id;
        app.documents()
            .delete(sub, document_id.clone().into())
            .await?;
        Ok(DocumentDeletePayload {
            deleted_document_id: document_id,
        })
    }

    async fn document_archive(
        &self,
        ctx: &Context<'_>,
        input: DocumentArchiveInput,
    ) -> async_graphql::Result<DocumentArchivePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let document_id = input.document_id;
        let document = app
            .documents()
            .archive(sub, document_id.clone().into())
            .await?;
        Ok(document.into())
    }

    async fn terms_template_create(
        &self,
        ctx: &Context<'_>,
        input: TermsTemplateCreateInput,
    ) -> async_graphql::Result<TermsTemplateCreatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let term_values = crate::terms::TermValues::builder()
            .annual_rate(input.annual_rate)
            .accrual_interval(input.accrual_interval)
            .incurrence_interval(input.incurrence_interval)
            .duration(input.duration)
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;

        let terms_template = app
            .terms_templates()
            .create_terms_template(sub, input.name, term_values)
            .await?;
        Ok(TermsTemplateCreatePayload::from(terms_template))
    }

    async fn terms_template_update(
        &self,
        ctx: &Context<'_>,
        input: TermsTemplateUpdateInput,
    ) -> async_graphql::Result<TermsTemplateUpdatePayload> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;

        let term_values = crate::terms::TermValues::builder()
            .annual_rate(input.annual_rate)
            .accrual_interval(input.accrual_interval)
            .incurrence_interval(input.incurrence_interval)
            .duration(input.duration)
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;
        let terms = app
            .terms_templates()
            .update_term_values(sub, TermsTemplateId::from(input.id), term_values)
            .await?;
        Ok(TermsTemplateUpdatePayload::from(terms))
    }
}
