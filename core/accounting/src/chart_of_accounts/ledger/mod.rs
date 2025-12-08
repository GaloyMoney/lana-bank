mod closing;
pub mod error;
mod template;

use cala_ledger::{
    AccountId, AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId,
    LedgerOperation, VelocityControlId, VelocityLimitId,
    account::{Account, NewAccount},
    account_set::{AccountSetMemberId, AccountSetUpdate, NewAccountSet},
    velocity::{NewBalanceLimit, NewLimit, NewVelocityControl, NewVelocityLimit, Params},
};
use chrono::NaiveDate;
use es_entity::{PaginatedQueryArgs, PaginatedQueryRet};
use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use closing::*;
use error::*;
use template::*;

use crate::{Chart, primitives::CalaTxId};

#[derive(Clone)]
pub struct ChartLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl ChartLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    #[record_error_severity]
    #[instrument(name = "chart_ledger.create_chart_root_account_set_in_op", skip(self, op, chart), fields(chart_id = %chart.id, chart_name = %chart.name))]
    pub async fn create_chart_root_account_set_in_op(
        &self,
        op: es_entity::DbOp<'_>,
        chart: &Chart,
    ) -> Result<(), ChartLedgerError> {
        let mut ledger_op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let new_account_set = NewAccountSet::builder()
            .id(chart.account_set_id)
            .journal_id(self.journal_id)
            .external_id(chart.id.to_string())
            .name(chart.name.clone())
            .description(chart.name.clone())
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");

        self.cala
            .account_sets()
            .create_in_op(&mut ledger_op, new_account_set)
            .await?;

        let control_id = self.create_close_control_in_op(&mut ledger_op).await?;

        self.cala
            .velocities()
            .attach_control_to_account_set_in_op(
                &mut ledger_op,
                control_id,
                chart.account_set_id,
                Params::new(),
            )
            .await?;

        ledger_op.commit().await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "chart_ledger.close_by_chart_root_account_set_as_of", 
        skip(self, op),
        fields(chart_id = tracing::field::Empty, closed_as_of = %closed_as_of)
    )]
    pub async fn close_by_chart_root_account_set_as_of(
        &self,
        op: es_entity::DbOp<'_>,
        closed_as_of: chrono::NaiveDate,
        chart_root_account_set_id: AccountSetId,
    ) -> Result<(), ChartLedgerError> {
        let mut tracking_account_set = self
            .cala
            .account_sets()
            .find(chart_root_account_set_id)
            .await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let mut account_set_metadata = tracking_account_set
            .values()
            .clone()
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));
        AccountingClosingMetadata::update_with_monthly_closing(
            &mut account_set_metadata,
            closed_as_of,
        );

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(account_set_metadata))
            .expect("Failed to serialize metadata");

        if tracking_account_set.update(update_values).did_execute() {
            self.cala
                .account_sets()
                .persist_in_op(&mut op, &mut tracking_account_set)
                .await?;
        }

        op.commit().await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "chart_ledger.attach_closing_controls_to_account_set_in_op",
        skip(self, op)
    )]
    async fn attach_closing_controls_to_account_set_in_op(
        &self,
        mut op: LedgerOperation<'_>,
        tracking_account_set_id: impl Into<AccountSetId> + std::fmt::Debug,
    ) -> Result<VelocityControlId, ChartLedgerError> {
        let tracking_account_set_id = tracking_account_set_id.into();
        let control_id = self.create_close_control_in_op(&mut op).await?;

        self.cala
            .velocities()
            .attach_control_to_account_set_in_op(
                &mut op,
                control_id,
                tracking_account_set_id,
                Params::new(),
            )
            .await?;

        op.commit().await?;

        Ok(control_id)
    }

    #[record_error_severity]
    #[instrument(name = "chart_ledger.create_close_control_in_op", skip(self, op))]
    async fn create_close_control_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
    ) -> Result<VelocityControlId, ChartLedgerError> {
        let monthly_cel_conditions = AccountingClosingMetadata::monthly_cel_conditions();
        let new_control = NewVelocityControl::builder()
            .id(VelocityControlId::new())
            .name("Account Closing")
            .description("Control to restrict posting to closed accounts")
            .condition(&monthly_cel_conditions)
            .build()
            .expect("build control");
        let control = self
            .cala
            .velocities()
            .create_control_in_op(op, new_control)
            .await?;

        // TODO: add_all to avoid n+1 ish issue
        let AccountClosingLimits {
            debit_settled: debit_settled_limit,
            debit_pending: debit_pending_limit,
            credit_settled: credit_settled_limit,
            credit_pending: credit_pending_limit,
        } = self.create_account_closing_limits_in_op(op).await?;

        self.cala
            .velocities()
            .add_limit_to_control_in_op(op, control.id(), debit_settled_limit.id())
            .await?;
        self.cala
            .velocities()
            .add_limit_to_control_in_op(op, control.id(), debit_pending_limit.id())
            .await?;
        self.cala
            .velocities()
            .add_limit_to_control_in_op(op, control.id(), credit_settled_limit.id())
            .await?;
        self.cala
            .velocities()
            .add_limit_to_control_in_op(op, control.id(), credit_pending_limit.id())
            .await?;

        Ok(control.id())
    }

    #[record_error_severity]
    #[instrument(
        name = "chart_ledger.create_account_closing_limits_in_op",
        skip(self, op)
    )]
    async fn create_account_closing_limits_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
    ) -> Result<AccountClosingLimits, ChartLedgerError> {
        let velocity = self.cala.velocities();

        let new_debit_settled_limit = NewVelocityLimit::builder()
            .id(VelocityLimitId::new())
            .name("Account Closed for debiting")
            .description("Ensures no transactions allowed")
            .window(vec![])
            .limit(
                NewLimit::builder()
                    .balance(vec![
                        NewBalanceLimit::builder()
                            .layer("SETTLED")
                            .amount("decimal('0')")
                            .enforcement_direction("DEBIT")
                            .build()
                            .expect("limit"),
                    ])
                    .build()
                    .expect("limit"),
            )
            .params(vec![])
            .build()
            .expect("build limit");

        let new_debit_pending_limit = NewVelocityLimit::builder()
            .id(VelocityLimitId::new())
            .name("Account Closed for debiting")
            .description("Ensures no transactions allowed")
            .window(vec![])
            .limit(
                NewLimit::builder()
                    .balance(vec![
                        NewBalanceLimit::builder()
                            .layer("PENDING")
                            .amount("decimal('0')")
                            .enforcement_direction("DEBIT")
                            .build()
                            .expect("limit"),
                    ])
                    .build()
                    .expect("limit"),
            )
            .params(vec![])
            .build()
            .expect("build limit");

        let new_credit_settled_limit = NewVelocityLimit::builder()
            .id(VelocityLimitId::new())
            .name("Account Closed for crediting")
            .description("Ensures no transactions allowed")
            .window(vec![])
            .limit(
                NewLimit::builder()
                    .balance(vec![
                        NewBalanceLimit::builder()
                            .layer("SETTLED")
                            .amount("decimal('0')")
                            .enforcement_direction("CREDIT")
                            .build()
                            .expect("limit"),
                    ])
                    .build()
                    .expect("limit"),
            )
            .params(vec![])
            .build()
            .expect("build limit");

        let new_credit_pending_limit = NewVelocityLimit::builder()
            .id(VelocityLimitId::new())
            .name("Account Closed for crediting")
            .description("Ensures no transactions allowed")
            .window(vec![])
            .limit(
                NewLimit::builder()
                    .balance(vec![
                        NewBalanceLimit::builder()
                            .layer("PENDING")
                            .amount("decimal('0')")
                            .enforcement_direction("CREDIT")
                            .build()
                            .expect("limit"),
                    ])
                    .build()
                    .expect("limit"),
            )
            .params(vec![])
            .build()
            .expect("build limit");

        // TODO: create_all to avoid n+1-ish issue
        let debit_settled_limit = velocity
            .create_limit_in_op(op, new_debit_settled_limit)
            .await?;
        let debit_pending_limit = velocity
            .create_limit_in_op(op, new_debit_pending_limit)
            .await?;
        let credit_settled_limit = velocity
            .create_limit_in_op(op, new_credit_settled_limit)
            .await?;
        let credit_pending_limit = velocity
            .create_limit_in_op(op, new_credit_pending_limit)
            .await?;

        Ok(AccountClosingLimits {
            debit_settled: debit_settled_limit,
            debit_pending: debit_pending_limit,
            credit_settled: credit_settled_limit,
            credit_pending: credit_pending_limit,
        })
    }

    #[record_error_severity]
    #[instrument(name = "chart_ledger.post_closing_transaction_in_op", skip(self, op))]
    pub async fn post_closing_transaction_in_op(
        &self,
        op: es_entity::DbOp<'_>,
        ledger_tx_id: CalaTxId,
        description: Option<String>,
        opened_as_of: NaiveDate,
        closed_as_of: NaiveDate,
        revenue_account_set_id: AccountSetId,
        cost_of_revenue_account_set_id: AccountSetId,
        expenses_account_set_id: AccountSetId,
        equity_retained_earnings_account_set_id: AccountSetId,
        equity_retained_losses_account_set_id: AccountSetId,
    ) -> Result<(), ChartLedgerError> {
        let (net_income, mut closing_entries) = self
            .get_closing_account_entry_params(
                revenue_account_set_id,
                cost_of_revenue_account_set_id,
                expenses_account_set_id,
                opened_as_of,
                closed_as_of,
            )
            .await?
            .to_closing_entries();
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        let equity_entry = self
            .create_equity_entry(
                &mut op,
                equity_retained_earnings_account_set_id,
                equity_retained_losses_account_set_id,
                net_income,
            )
            .await?;
        closing_entries.push(equity_entry);
        let closing_transaction_params = ClosingTransactionParams::new(
            self.journal_id,
            description.unwrap_or("Annual Closing".to_string()),
            closed_as_of,
            closing_entries,
        );
        let template = ClosingTransactionTemplate::init(
            &self.cala,
            closing_transaction_params.closing_entries.len(),
            "Fiscal Year Reference".to_string(),
        )
        .await?;
        self.cala
            .post_transaction_in_op(
                &mut op,
                ledger_tx_id,
                &template.code(),
                closing_transaction_params,
            )
            .await?;

        Ok(())
    }

    async fn get_closing_account_entry_params(
        &self,
        revenue_account_set_id: AccountSetId,
        cost_of_revenue_account_set_id: AccountSetId,
        expenses_account_set_id: AccountSetId,
        from: NaiveDate,
        until: NaiveDate,
    ) -> Result<ClosingAccountBalances, ChartLedgerError> {
        let revenue_accounts = self
            .find_all_accounts_by_parent_set_id(revenue_account_set_id)
            .await?;
        let expense_accounts = self
            .find_all_accounts_by_parent_set_id(expenses_account_set_id)
            .await?;
        let cost_of_revenue_accounts = self
            .find_all_accounts_by_parent_set_id(cost_of_revenue_account_set_id)
            .await?;

        let revenue_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&revenue_accounts, from, Some(until))
            .await?;
        let cost_of_revenue_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&cost_of_revenue_accounts, from, Some(until))
            .await?;
        let expenses_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&expense_accounts, from, Some(until))
            .await?;

        Ok(ClosingAccountBalances {
            revenue: revenue_account_balances,
            cost_of_revenue: cost_of_revenue_account_balances,
            expenses: expenses_account_balances,
        })
    }

    async fn find_all_accounts_by_parent_set_id(
        &self,
        parent_set_id: AccountSetId,
    ) -> Result<Vec<BalanceId>, ChartLedgerError> {
        let mut accounts: Vec<BalanceId> = Vec::new();

        let mut has_next_page = true;
        let mut after = None;

        while has_next_page {
            let PaginatedQueryRet {
                entities,
                has_next_page: next_page,
                end_cursor,
            } = self
                .cala
                .account_sets()
                .list_members_by_created_at(parent_set_id, PaginatedQueryArgs { first: 100, after })
                .await?;

            after = end_cursor;
            has_next_page = next_page;

            for member in entities {
                match member.id {
                    AccountSetMemberId::Account(account_id) => {
                        accounts.push((self.journal_id, account_id, Currency::USD));
                    }
                    AccountSetMemberId::AccountSet(account_set_id) => {
                        let nested_accounts =
                            Box::pin(self.find_all_accounts_by_parent_set_id(account_set_id))
                                .await?;
                        accounts.extend(nested_accounts);
                    }
                }
            }
        }
        Ok(accounts)
    }

    async fn create_equity_entry(
        &self,
        op: &mut LedgerOperation<'_>,
        equity_retained_earnings_account_set_id: AccountSetId,
        equity_retained_losses_account_set_id: AccountSetId,
        net_earnings: Decimal,
    ) -> Result<ClosingAccountEntry, ChartLedgerError> {
        let account = if net_earnings >= Decimal::ZERO {
            self.create_account_in_op(
                op,
                DebitOrCredit::Credit,
                equity_retained_earnings_account_set_id,
            )
            .await?
        } else {
            self.create_account_in_op(
                op,
                DebitOrCredit::Debit,
                equity_retained_losses_account_set_id,
            )
            .await?
        };

        Ok(ClosingAccountEntry {
            account_id: account.id.into(),
            currency: Currency::USD,
            amount: net_earnings.abs(),
            direction: account.values().normal_balance_type,
        })
    }

    async fn create_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        normal_balance_type: DebitOrCredit,
        parent_account_set: AccountSetId,
    ) -> Result<Account, ChartLedgerError> {
        let id = AccountId::new();
        // TODO: `name` as input parameter (`FiscalYear` `reference`?)
        let new_ledger_account = NewAccount::builder()
            .id(id)
            .name("Retained Earnings")
            .code(id.to_string())
            .normal_balance_type(normal_balance_type)
            .build()
            .expect("Could not build new account for annual close net income transfer entry");
        let ledger_account = self
            .cala
            .accounts()
            .create_in_op(op, new_ledger_account)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(op, parent_account_set, ledger_account.id)
            .await?;

        Ok(ledger_account)
    }
}
