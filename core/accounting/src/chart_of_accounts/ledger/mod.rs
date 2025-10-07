mod closing;
pub mod error;

use std::collections::HashMap;
use rust_decimal::Decimal;

use cala_ledger::{
    AccountId, AccountSetId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
    VelocityControlId, VelocityLimitId,
    account_set::{AccountSetUpdate, NewAccountSet},
    balance::AccountBalance,
    velocity::{
        NewBalanceLimit, NewLimit, NewVelocityControl, NewVelocityLimit, Params, VelocityLimit,
    },
};

use closing::*;
use error::*;

use crate::{primitives::TransactionEntrySpec, AccountIdOrCode};

use crate::Chart;

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

    pub async fn create_chart_root_account_set_in_op(
        &self,
        op: es_entity::DbOp<'_>,
        chart: &Chart,
    ) -> Result<(), ChartLedgerError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let new_account_set = NewAccountSet::builder()
            .id(chart.id)
            .journal_id(self.journal_id)
            .external_id(chart.id.to_string())
            .name(chart.name.clone())
            .description(chart.name.clone())
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");
        let mut chart_account_set = self
            .cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;

        let control_id = self
            .create_monthly_close_control_with_limits_in_op(&mut op)
            .await?;
        self.cala
            .velocities()
            .attach_control_to_account_set_in_op(
                &mut op,
                control_id,
                chart_account_set.id(),
                Params::new(),
            )
            .await?;

        let mut metadata = chart_account_set
            .values()
            .clone()
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));
        AccountingClosingMetadata::update_metadata(
            &mut metadata,
            chart.monthly_closing.closed_as_of,
        );

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(metadata))
            .expect("Failed to serialize metadata");

        chart_account_set.update(update_values);
        self.cala
            .account_sets()
            .persist_in_op(&mut op, &mut chart_account_set)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn monthly_close_chart_as_of(
        &self,
        op: es_entity::DbOp<'_>,
        chart_root_account_set_id: impl Into<AccountSetId>,
        closed_as_of: chrono::NaiveDate,
    ) -> Result<(), ChartLedgerError> {
        let id = chart_root_account_set_id.into();
        let mut chart_account_set = self.cala.account_sets().find(id).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let mut metadata = chart_account_set
            .values()
            .clone()
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));
        AccountingClosingMetadata::update_metadata(&mut metadata, closed_as_of);

        let mut update_values = AccountSetUpdate::default();
        update_values
            .metadata(Some(metadata))
            .expect("Failed to serialize metadata");

        chart_account_set.update(update_values);
        self.cala
            .account_sets()
            .persist_in_op(&mut op, &mut chart_account_set)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn prepare_annual_closing_transaction(
        &self,
        _op: es_entity::DbOp<'_>,
        chart_root_account_set_id: impl Into<AccountSetId>,
        // TODO: Check types to use for ChartLedger params.
        _revenue_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
        _expense_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
        _cost_of_revenue_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
        _retained_earnings_account_set: AccountSetId,
        _retained_losses_account_set: AccountSetId,
    ) -> Result<Vec<TransactionEntrySpec>, ChartLedgerError> {
        let _id = chart_root_account_set_id.into();
        // TODO: Use a transaction template to create the entries in Cala that should -
        // (1) debit the Revenue Account(Set members and aggregate balance)
        // (2) credit the Cost of Revenue Account(Set members and aggregate balance)
        // (3) credit the Expenses Account(Set members and aggregate balance)
        // (4) credit/debit (depending on the net amount from 1,2, and 3) the Equity AccountSet (Patrimonios > Utilidades ???)

        // TODO: Create and attach accounts to target AccountSet (both under the Equity AccountCode).

        Ok(vec![])
    }

    fn _create_annual_close_offset_entries(
        &self, 
        accounts_by_code: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
    ) -> Result<Vec<TransactionEntrySpec>, ChartLedgerError> {

        let mut entries = Vec::new();
        let mut net: Decimal = Decimal::from(0);

        for ((journal_id, account_id, currency), bal_details) in accounts_by_code.iter() {
            let amt = bal_details.settled();

            // TODO: Other considerations here for `pending` or `encumbrance`?
            // let entry = TransactionEntrySpec {
            //     // TODO: go from (Cala)AccountId to LedgerAccountId to satisfy AccountIdOrCode.
            //     //account_id: AccountIdOrCode::Id(account_id.into()),
            //     currency: currency.clone(),
            //     amount: amt,
            //     // TODO: Add a parameter for this field?
            //     description: "Annual Close Offset".to_string(),
            //     // TODO: How should we determine the direction - do we need to know the AccountCode
            //     // we are operating on?
            //     direction: DebitOrCredit::Debit,
            // };
        }
        Ok(entries)
    }

    async fn create_monthly_close_control_with_limits_in_op(
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

        // TODO: add_all to avoid n+1-ish issue
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
}

struct AccountClosingLimits {
    debit_settled: VelocityLimit,
    debit_pending: VelocityLimit,
    credit_settled: VelocityLimit,
    credit_pending: VelocityLimit,
}
