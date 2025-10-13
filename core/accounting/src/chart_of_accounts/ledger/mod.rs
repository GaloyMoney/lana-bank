mod closing;

mod template;
use template::*;

pub mod error;

use cala_ledger::{
    AccountSetId, CalaLedger, DebitOrCredit, JournalId, LedgerOperation, VelocityControlId,
    VelocityLimitId,
    account_set::{AccountSetUpdate, NewAccountSet},
    velocity::{
        NewBalanceLimit, NewLimit, NewVelocityControl, NewVelocityLimit, Params, VelocityLimit,
    },
};

use closing::*;
use error::*;

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

    pub async fn execute_annual_close_transaction(
        &self,
        op: es_entity::DbOp<'_>,
        chart_root_account_set_id: impl Into<AccountSetId>,
        closed_as_of: chrono::NaiveDate,
    ) -> Result<(), ChartLedgerError> {
        let mut op = self
        .cala
        .ledger_operation_from_db_op(op.with_db_time().await?);

        // TODO: Move logic to build params based on `chart_root_account_set_id`

        // let template =
        //     AnnualClosingTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        // self.cala
        //     .post_transaction_in_op(&mut op, tx_id, &template.code(), params)
        //     .await?;

        // op.commit().await?;
        Ok(())
    }
}

struct AccountClosingLimits {
    debit_settled: VelocityLimit,
    debit_pending: VelocityLimit,
    credit_settled: VelocityLimit,
    credit_pending: VelocityLimit,
}
