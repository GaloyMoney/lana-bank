mod closing_balances;
pub(crate) mod closing_metadata;
pub mod error;
mod template;

use cala_ledger::{
    AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId, TxTemplateId,
    VelocityControlId, VelocityLimitId,
    account::Account,
    account_set::{AccountSetMemberId, AccountSetUpdate, NewAccountSet},
    tx_template::{
        NewTxTemplate, NewTxTemplateEntry, NewTxTemplateTransaction, error::TxTemplateError,
    },
    velocity::{NewBalanceLimit, NewLimit, NewVelocityControl, NewVelocityLimit, Params},
};

use es_entity::{PaginatedQueryArgs, PaginatedQueryRet};

use tracing::instrument;
use tracing_macros::record_error_severity;

pub(super) use closing_balances::*;
use closing_metadata::*;
use error::*;
use template::*;

use crate::{Chart, ClosingTxDetails};

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
        op: &mut es_entity::DbOp<'_>,
        chart: &Chart,
    ) -> Result<(), ChartLedgerError> {
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
            .create_in_op(op, new_account_set)
            .await?;

        let control_id = self.create_close_control_in_op(op).await?;

        self.cala
            .velocities()
            .attach_control_to_account_set_in_op(
                op,
                control_id,
                chart.account_set_id,
                Params::new(),
            )
            .await?;
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
        op: &mut es_entity::DbOp<'_>,
        closed_as_of: chrono::NaiveDate,
        chart_root_account_set_id: AccountSetId,
    ) -> Result<(), ChartLedgerError> {
        let mut tracking_account_set = self
            .cala
            .account_sets()
            .find_in_op(op, chart_root_account_set_id)
            .await?;

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
                .persist_in_op(op, &mut tracking_account_set)
                .await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "chart_ledger.attach_closing_controls_to_account_set_in_op",
        skip(self, op)
    )]
    async fn attach_closing_controls_to_account_set_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        tracking_account_set_id: impl Into<AccountSetId> + std::fmt::Debug,
    ) -> Result<VelocityControlId, ChartLedgerError> {
        let tracking_account_set_id = tracking_account_set_id.into();
        let control_id = self.create_close_control_in_op(op).await?;

        self.cala
            .velocities()
            .attach_control_to_account_set_in_op(
                op,
                control_id,
                tracking_account_set_id,
                Params::new(),
            )
            .await?;
        Ok(control_id)
    }

    #[record_error_severity]
    #[instrument(name = "chart_ledger.create_close_control_in_op", skip(self, op))]
    async fn create_close_control_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
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
        op: &mut es_entity::DbOp<'_>,
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
    #[instrument(name = "chart_ledger.post_closing_transaction", skip(self, op))]
    pub async fn post_closing_transaction(
        &self,
        op: &mut es_entity::DbOp<'_>,
        ClosingTxParentIdsAndDetails {
            net_income_parent_ids,
            retained_earnings_parent_ids,
            tx_details,
        }: ClosingTxParentIdsAndDetails,
    ) -> Result<(), ChartLedgerError> {
        let balances = self
            .find_all_profit_and_loss_statement_effective_balances(
                net_income_parent_ids,
                &tx_details,
            )
            .await?;

        let net_income_recipient_account = if let Some(retained_earnings_details) = balances
            .retained_earnings_new_account(
                tx_details.retained_earnings_account_name(),
                retained_earnings_parent_ids,
            ) {
            Some(
                self.create_child_account_in_op(op, retained_earnings_details)
                    .await?,
            )
        } else {
            None
        };

        let ClosingTxDetails {
            tx_id,
            effective_balances_until,
            description,
            ..
        } = tx_details;

        let closing_transaction_params = ClosingTransactionParams::new(
            self.journal_id,
            description,
            effective_balances_until,
            balances.entries_params(net_income_recipient_account),
        );
        let template_code = self
            .find_or_create_template(op, &closing_transaction_params)
            .await?;

        self.cala
            .post_transaction_in_op(op, tx_id, &template_code, closing_transaction_params)
            .await?;
        Ok(())
    }

    async fn find_all_profit_and_loss_statement_effective_balances(
        &self,
        NetIncomeAccountSetIds {
            revenue: revenue_account_set_id,
            cost_of_revenue: cost_of_revenue_account_set_id,
            expenses: expenses_account_set_id,
        }: NetIncomeAccountSetIds,
        ClosingTxDetails {
            effective_balances_from: from,
            effective_balances_until: until,
            ..
        }: &ClosingTxDetails,
    ) -> Result<ClosingProfitAndLossAccountBalances, ChartLedgerError> {
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
            .find_all_in_range(&revenue_accounts, *from, Some(*until))
            .await?;
        let cost_of_revenue_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&cost_of_revenue_accounts, *from, Some(*until))
            .await?;
        let expenses_account_balances = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&expense_accounts, *from, Some(*until))
            .await?;
        Ok(ClosingProfitAndLossAccountBalances {
            revenue: revenue_account_balances.into(),
            cost_of_revenue: cost_of_revenue_account_balances.into(),
            expenses: expenses_account_balances.into(),
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
                        // TODO: Lookup the account currency using `account_id`?
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

    async fn create_child_account_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        NewAccountDetails {
            new_account: new_ledger_account,
            parent_account_set_id,
        }: NewAccountDetails,
    ) -> Result<Account, ChartLedgerError> {
        let ledger_account = self
            .cala
            .accounts()
            .create_in_op(op, new_ledger_account)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(op, parent_account_set_id, ledger_account.id)
            .await?;

        Ok(ledger_account)
    }

    async fn find_or_create_template(
        &self,
        op: &mut es_entity::DbOp<'_>,
        params: &ClosingTransactionParams,
    ) -> Result<String, TxTemplateError> {
        let n_entries = params.entries_params.len();
        let code = params.template_code();

        let mut entries = vec![];
        for i in 0..n_entries {
            entries.push(
                NewTxTemplateEntry::builder()
                    .entry_type(params.tx_entry_type(i))
                    .account_id(format!("params.{}", EntryParams::account_id_param_name(i)))
                    .units(format!("params.{}", EntryParams::amount_param_name(i)))
                    .currency(format!("params.{}", EntryParams::currency_param_name(i)))
                    .layer(format!("params.{}", EntryParams::layer_param_name(i)))
                    .direction(format!("params.{}", EntryParams::direction_param_name(i)))
                    .build()
                    .expect("Couldn't build entry for ClosingTransactionTemplate"),
            );
        }

        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .description("params.description")
            .effective("params.effective")
            .metadata("params.meta")
            .build()
            .expect("Couldn't build TxInput for ClosingTransactionTemplate");

        let params = ClosingTransactionParams::defs(n_entries);
        let new_template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(&code)
            .transaction(tx_input)
            .entries(entries)
            .params(params)
            .description(format!(
                "Template to execute a closing transaction with {} entries.",
                n_entries
            ))
            .build()
            .expect("Couldn't build template for ClosingTransactionTemplate");
        match self
            .cala
            .tx_templates()
            .create_in_op(op, new_template)
            .await
        {
            Err(TxTemplateError::DuplicateCode) => Ok(code),
            Err(e) => Err(e),
            Ok(template) => Ok(template.into_values().code),
        }
    }
}
