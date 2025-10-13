mod closing;
pub mod error;

use rust_decimal::Decimal;
use std::collections::HashMap;

use cala_ledger::{
    account::NewAccount, account_set::{AccountSetUpdate, NewAccountSet, AccountSetMemberId}, balance::AccountBalance, velocity::{
        NewBalanceLimit, NewLimit, NewVelocityControl, NewVelocityLimit, Params, VelocityLimit,
    }, AccountId, AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation, VelocityControlId, VelocityLimitId
};

use closing::*;
use error::*;

use crate::{
    CHART_OF_ACCOUNTS_ENTITY_TYPE, EntityRef, LedgerAccountId, primitives::TransactionEntrySpec,
};

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

    pub async fn prepare_annual_closing_entries(
        &self,
        op: es_entity::DbOpWithTime<'_>,
        // TODO: Check types to use for ChartLedger params.
        revenue_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
        expense_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
        cost_of_revenue_accounts: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
        retained_earnings_account_set: AccountSetId,
        retained_losses_account_set: AccountSetId,
    ) -> Result<Vec<TransactionEntrySpec>, ChartLedgerError> {
        let (revenue_offset_entries, net_revenue) =
            self.create_annual_close_offset_entries(DebitOrCredit::Credit, None, revenue_accounts);
        let (expense_offset_entries, net_expenses) =
            self.create_annual_close_offset_entries(DebitOrCredit::Debit, None, expense_accounts);
        let (cost_of_revenue_offset_entries, net_cost_of_revenue) = self
            .create_annual_close_offset_entries(
                DebitOrCredit::Debit,
                None,
                cost_of_revenue_accounts,
            );
        let mut all_entries = Vec::new();
        all_entries.extend(revenue_offset_entries);
        all_entries.extend(expense_offset_entries);
        all_entries.extend(cost_of_revenue_offset_entries);

        let retained_earnings = net_revenue - net_expenses - net_cost_of_revenue;
        let mut op = self.cala.ledger_operation_from_db_op(op);
        let equity_entry = self
            .create_annual_close_equity_target(
                &mut op,
                retained_earnings,
                retained_earnings_account_set,
                retained_losses_account_set,
            )
            .await?;
        all_entries.extend(vec![equity_entry.clone()]);
        op.commit().await?;

        Ok(all_entries)
    }

    // TODO: Rename / refactor.
    async fn create_annual_close_equity_target(
        &self,
        op: &mut LedgerOperation<'_>,
        net_earnings: Decimal,
        retained_earnings_account_set: AccountSetId,
        retained_losses_account_set: AccountSetId,
    ) -> Result<TransactionEntrySpec, ChartLedgerError> {
        let (direction, parent_account_set, reference) = if net_earnings > Decimal::ZERO {
            (
                DebitOrCredit::Credit,
                retained_earnings_account_set,
                "retained_earnings",
            )
        } else {
            (
                DebitOrCredit::Debit,
                retained_losses_account_set,
                "retained_losses",
            )
        };

        // TODO: Evaluate where these params should be sourced from.
        let account_id = self
            .create_annual_close_equity_account(
                op,
                reference,
                "Annual Close Net Income",
                "Annual Close Net Income",
                direction,
                parent_account_set,
            )
            .await?;
        let ledger_account_id = LedgerAccountId::from(account_id);
        Ok(TransactionEntrySpec {
            account_id: ledger_account_id,
            // TODO: Make currency a param?
            currency: Currency::USD,
            amount: net_earnings,
            // TODO: Make description a param?
            description: "Annual Close Net Income to Equity".to_string(),
            direction,
        })
    }

    async fn create_annual_close_equity_account(
        &self,
        op: &mut cala_ledger::LedgerOperation<'_>,
        reference: &str,
        name: &str,
        description: &str,
        normal_balance_type: DebitOrCredit,
        parent_account_set: AccountSetId,
    ) -> Result<AccountId, ChartLedgerError> {
        let id = AccountId::new();
        let entity_ref = EntityRef::new(CHART_OF_ACCOUNTS_ENTITY_TYPE, id);
        self.create_annual_close_equity_account_in_op(
            op,
            id,
            reference,
            name,
            description,
            entity_ref,
            normal_balance_type,
            parent_account_set,
        )
        .await
    }

    async fn create_annual_close_equity_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        id: impl Into<AccountId>,
        reference: &str,
        name: &str,
        description: &str,
        entity_ref: EntityRef,
        normal_balance_type: DebitOrCredit,
        parent_account_set: AccountSetId,
        // TODO: Metadata?
    ) -> Result<AccountId, ChartLedgerError> {
        let id = id.into();
        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference)
            .name(name)
            .description(description)
            .code(id.to_string())
            .normal_balance_type(normal_balance_type)
            .metadata(serde_json::json!({"entity_ref": entity_ref}))
            .expect("Could not add metadata")
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

        Ok(ledger_account.id)
    }

    fn create_annual_close_offset_entries(
        &self,
        // TODO: Can we make this assumption across a category
        // or do we need to expose `AccountBalance.balance_type` from Cala?
        // https://www.twisp.com/docs/accounting-core/chart-of-accounts#credit-normal-and-debit-normal
        normal_balance_type: DebitOrCredit,
        description: Option<String>,
        accounts_by_code: HashMap<(JournalId, AccountId, Currency), AccountBalance>,
    ) -> (Vec<TransactionEntrySpec>, Decimal) {
        let mut entries = Vec::new();
        let mut net: Decimal = Decimal::from(0);
        for ((_journal_id, account_id, currency), bal_details) in accounts_by_code.iter() {
            // TODO: Other considerations here for `pending` or `encumbrance`?
            let amt = bal_details.settled();
            if amt == Decimal::ZERO {
                continue;
            }
            net += amt;
            // TODO: - Related to note on `normal_balance_type` param.
            let direction = if normal_balance_type == DebitOrCredit::Debit {
                DebitOrCredit::Credit
            } else {
                DebitOrCredit::Debit
            };
            // TODO: go from (Cala)AccountId to LedgerAccountId to satisfy AccountIdOrCode properly.
            let ledger_account_id = LedgerAccountId::from(*account_id);
            let entry = TransactionEntrySpec {
                account_id: ledger_account_id,
                currency: currency.clone(),
                amount: amt,
                description: description
                    .clone()
                    .unwrap_or("Annual Close Offset".to_string()),
                direction: direction,
            };
            entries.push(entry);
        }
        (entries, net)
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

    pub async fn find_all_accounts_by_parent_set_id(
        &self,
        journal_id: JournalId,
        parent_set_id: AccountSetId,
    ) -> Result<Vec<BalanceId>, ChartLedgerError> {
        let mut accounts: Vec<BalanceId> = Vec::new();
        // TODO: Does this require pagination or should we use a non default value?
        let members = self.cala
            .account_sets()
            .list_members_by_created_at(
                parent_set_id, 
                Default::default()
            )
            .await?;
        for member in members.entities {
            match member.id {
                AccountSetMemberId::Account(account_id) => {
                    accounts.push((journal_id, account_id.clone(), Currency::USD));
                }
                AccountSetMemberId::AccountSet(account_set_id) => {
                    let nested_accounts = Box::pin(self.find_all_accounts_by_parent_set_id(journal_id, account_set_id)).await?;
                    accounts.extend(nested_accounts);
                }
            }
        }
        Ok(accounts)
    }
}

struct AccountClosingLimits {
    debit_settled: VelocityLimit,
    debit_pending: VelocityLimit,
    credit_settled: VelocityLimit,
    credit_pending: VelocityLimit,
}
