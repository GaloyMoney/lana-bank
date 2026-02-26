use async_graphql::{Context, MergedObject, MergedSubscription, Object, types::connection::*};

use admin_graphql_access::{AccessMutation, AccessQuery};
use admin_graphql_accounting::{AccountingMutation, AccountingQuery, AccountingSubscription};
use admin_graphql_audit::AuditQuery;
use admin_graphql_config::{ConfigMutation, ConfigQuery};
use admin_graphql_contracts::{ContractsMutation, ContractsQuery};
use admin_graphql_credit::{CreditMutation, CreditQuery, CreditSubscription};
use admin_graphql_custody::{CustodyMutation, CustodyQuery};
use admin_graphql_customer::{CustomerMutation, CustomerQuery};
use admin_graphql_deposit::{DepositMutation, DepositQuery};
use admin_graphql_documents::{DocumentsMutation, DocumentsQuery};
use admin_graphql_governance::{GovernanceMutation, GovernanceQuery};
use admin_graphql_price::{PriceQuery, PriceSubscription};
use admin_graphql_reports::{ReportsMutation, ReportsQuery, ReportsSubscription};
use admin_graphql_session::{SessionMutation, SessionQuery};

use lana_app::accounting_init::constants::{
    BALANCE_SHEET_NAME, PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME,
};

use crate::primitives::*;

use super::{
    accounting::*, credit_facility::*, customer::*, deposit::*, loader::*, prospect::*,
    public_id::*, withdrawal::*,
};

#[derive(MergedObject, Default)]
pub struct Query(
    pub AccessQuery,
    pub AccountingQuery,
    pub AuditQuery,
    pub ConfigQuery,
    pub ContractsQuery,
    pub CustomerQuery,
    pub CreditQuery,
    pub CustodyQuery,
    pub DepositQuery,
    pub DocumentsQuery,
    pub GovernanceQuery,
    pub PriceQuery,
    pub ReportsQuery,
    pub SessionQuery,
    pub BaseQuery,
);

#[derive(Default)]
pub struct BaseQuery;

#[Object]
impl BaseQuery {
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

    async fn public_id_target(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<PublicIdTarget>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let Some(public_id) = app.public_ids().find_by_id(id).await? else {
            return Ok(None);
        };

        let res = match public_id.target_type.as_str() {
            "customer" => app
                .customers()
                .find_by_id(sub, CustomerId::from(public_id.target_id))
                .await?
                .map(Customer::from)
                .map(PublicIdTarget::Customer),
            "deposit_account" => app
                .deposits()
                .find_account_by_id(sub, DepositAccountId::from(public_id.target_id))
                .await?
                .map(DepositAccount::from)
                .map(PublicIdTarget::DepositAccount),
            "deposit" => app
                .deposits()
                .find_deposit_by_id(sub, DepositId::from(public_id.target_id))
                .await?
                .map(Deposit::from)
                .map(PublicIdTarget::Deposit),
            "withdrawal" => app
                .deposits()
                .find_withdrawal_by_id(sub, WithdrawalId::from(public_id.target_id))
                .await?
                .map(Withdrawal::from)
                .map(PublicIdTarget::Withdrawal),
            "credit_facility" => app
                .credit()
                .facilities()
                .find_by_id(sub, CreditFacilityId::from(public_id.target_id))
                .await?
                .map(CreditFacility::from)
                .map(PublicIdTarget::CreditFacility),
            "disbursal" => app
                .credit()
                .disbursals()
                .find_by_id(sub, DisbursalId::from(public_id.target_id))
                .await?
                .map(CreditFacilityDisbursal::from)
                .map(PublicIdTarget::CreditFacilityDisbursal),
            "prospect" => app
                .customers()
                .find_prospect_by_id(sub, ProspectId::from(public_id.target_id))
                .await?
                .map(Prospect::from)
                .map(PublicIdTarget::Prospect),
            _ => None,
        };
        Ok(res)
    }
}

#[derive(MergedObject, Default)]
pub struct Mutation(
    pub AccessMutation,
    pub AccountingMutation,
    pub ConfigMutation,
    pub ContractsMutation,
    pub CustomerMutation,
    pub CreditMutation,
    pub CustodyMutation,
    pub DepositMutation,
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
}

#[derive(MergedSubscription, Default)]
pub struct Subscription(
    pub AccountingSubscription,
    pub CreditSubscription,
    pub PriceSubscription,
    pub ReportsSubscription,
);
