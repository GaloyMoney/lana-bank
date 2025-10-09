pub mod chart_node;
mod entity;
mod import;

pub mod error;
pub mod ledger;
mod repo;
pub mod tree;

use es_entity::Idempotent;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::{AccountSetId, BalanceId, CalaLedger, Currency, account::Account};

use crate::{
    TransactionEntrySpec,
    primitives::{
        AccountCode, AccountIdOrCode, AccountName, AccountSpec, CalaAccountSetId, CalaJournalId,
        ChartId, CoreAccountingAction, CoreAccountingObject, LedgerAccountId,
    },
};

#[cfg(feature = "json-schema")]
pub use chart_node::ChartNodeEvent;
#[cfg(feature = "json-schema")]
pub use entity::ChartEvent;
pub(super) use entity::*;
pub use entity::{Chart, PeriodClosing};
use error::*;
use import::{
    BulkAccountImport, BulkImportResult,
    csv::{CsvParseError, CsvParser},
};
use ledger::*;
pub(super) use repo::*;

pub struct ChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    repo: ChartRepo,
    chart_ledger: ChartLedger,
    cala: CalaLedger, // TODO: move calls into chart ledger
    authz: Perms,
    journal_id: CalaJournalId,
}

impl<Perms> Clone for ChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            chart_ledger: self.chart_ledger.clone(),
            cala: self.cala.clone(),
            authz: self.authz.clone(),
            journal_id: self.journal_id,
        }
    }
}

impl<Perms> ChartOfAccounts<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
    ) -> Self {
        let chart_of_account = ChartRepo::new(pool);
        let chart_ledger = ChartLedger::new(cala, journal_id);

        Self {
            repo: chart_of_account,
            chart_ledger,
            cala: cala.clone(),
            authz: authz.clone(),
            journal_id,
        }
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.create_chart",
        skip(self),
        err
    )]
    pub async fn create_chart(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
        reference: String,
        first_period_opened_as_of: chrono::NaiveDate,
    ) -> Result<Chart, ChartOfAccountsError> {
        let id = ChartId::new();

        let mut op = self.repo.begin_op().await?;
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_CREATE,
            )
            .await?;

        let new_chart = NewChart::builder()
            .id(id)
            .account_set_id(id)
            .name(name)
            .reference(reference)
            .first_period_opened_as_of(first_period_opened_as_of)
            .build()
            .expect("Could not build new chart of accounts");

        let chart = self.repo.create_in_op(&mut op, new_chart).await?;

        self.chart_ledger
            .create_chart_root_account_set_in_op(op, &chart)
            .await?;

        Ok(chart)
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.import_from_csv",
        skip(self, data),
        err
    )]
    pub async fn import_from_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartId> + std::fmt::Debug,
        data: impl AsRef<str>,
    ) -> Result<(Chart, Option<Vec<CalaAccountSetId>>), ChartOfAccountsError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_IMPORT_ACCOUNTS,
            )
            .await?;
        let mut chart = self.repo.find_by_id(id).await?;

        let data = data.as_ref().to_string();
        let account_specs = CsvParser::new(data).account_specs()?;

        let BulkImportResult {
            new_account_sets,
            new_account_set_ids,
            new_connections,
        } = BulkAccountImport::new(&mut chart, self.journal_id).import(account_specs);

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        self.cala
            .account_sets()
            .create_all_in_op(&mut op, new_account_sets)
            .await?;

        for (parent, child) in new_connections {
            self.cala
                .account_sets()
                .add_member_in_op(&mut op, parent, child)
                .await?;
        }
        op.commit().await?;

        let new_account_set_ids = &chart
            .trial_balance_account_ids_from_new_accounts(&new_account_set_ids)
            .collect::<Vec<_>>();

        Ok((chart, Some(new_account_set_ids.clone())))
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.close_monthly",
        skip(self,),
        err
    )]
    pub async fn close_monthly(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<Chart, ChartOfAccountsError> {
        let id = id.into();
        let mut chart = self.repo.find_by_id(id).await?;

        let now = crate::time::now();
        let closed_as_of_date =
            if let Idempotent::Executed(date) = chart.close_last_monthly_period(now)? {
                date
            } else {
                return Ok(chart);
            };

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        self.chart_ledger
            .monthly_close_chart_as_of(op, chart.id, closed_as_of_date)
            .await?;

        Ok(chart)
    }

    pub async fn create_annual_closing_entries(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<Vec<TransactionEntrySpec>, ChartOfAccountsError> {
        let id = id.into();
        let chart = self.repo.find_by_id(id).await?;

        if !chart.is_last_monthly_period_closed() {
            return Err(ChartOfAccountsError::AccountPeriodAnnualCloseNotReady);
        }
        // TODO: Where should we get these codes from? "6", "7", "8" intending to capture
        // "Revenue", "Cost of Revenue", "Expenses". May need to add an Account to
        // the "Equity" account set as a part of this process also, so. Note, there
        // is a TODO inside `is_ready_for_annual_closing_transaction` that also mentions
        // a possible need for additional config (or firm assumptions).
        let revenue_parent_code = "3".parse::<AccountCode>().unwrap();
        let revenue_set_id = chart.account_set_id_from_code(&revenue_parent_code)?;

        let cost_of_revenue_parent_code = "7".parse::<AccountCode>().unwrap();
        let cost_of_revenue_set_id =
            chart.account_set_id_from_code(&cost_of_revenue_parent_code)?;

        let expenses_parent_code = "8".parse::<AccountCode>().unwrap();
        let expenses_set_id = chart.account_set_id_from_code(&expenses_parent_code)?;

        // TODO: These profit/loss destination AccountSets must also be configured but slightly differently than the ProfitAndLoss (top-level) AccountSets.
        let retained_earnings_set_id = AccountSetId::new();
        let retained_losses_set_id = AccountSetId::new();

        // TODO: Abstract or condense the account collection process across
        // Revenue, Cost of Revenue, and Expenses top-level AccountSets.
        let mut revenue_accounts: Vec<BalanceId> = Vec::new();
        // TODO: Does this require pagination or should we use a non default value?
        let revenue_account_sets = self
            .cala
            .account_sets()
            .list_members_by_created_at(revenue_set_id, Default::default())
            .await?;

        for member in &revenue_account_sets.entities {
            match &member.id {
                cala_ledger::account_set::AccountSetMemberId::Account(account_id) => {
                    // TODO: Can we assume Currency::USD here i.e. thinking BTC may be for collateral only?
                    revenue_accounts.push((self.journal_id, account_id.clone(), Currency::USD));
                }
                cala_ledger::account_set::AccountSetMemberId::AccountSet(account_set_id) => {
                    let mut sets_to_process = vec![*account_set_id];

                    while !sets_to_process.is_empty() {
                        let current_level_sets = std::mem::take(&mut sets_to_process);

                        for set_id in current_level_sets {
                            let members = self
                                .cala
                                .account_sets()
                                .list_members_by_created_at(set_id, Default::default())
                                .await?
                                .entities;

                            for member in members {
                                match member.id {
                                    cala_ledger::account_set::AccountSetMemberId::Account(
                                        account_id,
                                    ) => {
                                        revenue_accounts.push((
                                            self.journal_id,
                                            account_id.clone(),
                                            Currency::USD,
                                        ));
                                    }
                                    cala_ledger::account_set::AccountSetMemberId::AccountSet(
                                        nested_set_id,
                                    ) => {
                                        sets_to_process.push(nested_set_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut expense_accounts: Vec<BalanceId> = Vec::new();
        let expenses_account_sets = self
            .cala
            .account_sets()
            .list_members_by_created_at(expenses_set_id, Default::default())
            .await?;

        for member in &expenses_account_sets.entities {
            match &member.id {
                cala_ledger::account_set::AccountSetMemberId::Account(account_id) => {
                    expense_accounts.push((self.journal_id, account_id.clone(), Currency::USD));
                }
                cala_ledger::account_set::AccountSetMemberId::AccountSet(account_set_id) => {
                    let mut sets_to_process = vec![*account_set_id];

                    while !sets_to_process.is_empty() {
                        let current_level_sets = std::mem::take(&mut sets_to_process);

                        for set_id in current_level_sets {
                            let members = self
                                .cala
                                .account_sets()
                                .list_members_by_created_at(set_id, Default::default())
                                .await?
                                .entities;

                            for member in members {
                                match member.id {
                                    cala_ledger::account_set::AccountSetMemberId::Account(
                                        account_id,
                                    ) => {
                                        expense_accounts.push((
                                            self.journal_id,
                                            account_id.clone(),
                                            Currency::USD,
                                        ));
                                    }
                                    cala_ledger::account_set::AccountSetMemberId::AccountSet(
                                        nested_set_id,
                                    ) => {
                                        sets_to_process.push(nested_set_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut cost_of_revenue_accounts: Vec<BalanceId> = Vec::new();
        let cost_of_revenue_account_sets = self
            .cala
            .account_sets()
            .list_members_by_created_at(cost_of_revenue_set_id, Default::default())
            .await?;
        for member in &cost_of_revenue_account_sets.entities {
            match &member.id {
                cala_ledger::account_set::AccountSetMemberId::Account(account_id) => {
                    cost_of_revenue_accounts.push((
                        self.journal_id,
                        account_id.clone(),
                        Currency::USD,
                    ));
                }
                cala_ledger::account_set::AccountSetMemberId::AccountSet(account_set_id) => {
                    let mut sets_to_process = vec![*account_set_id];

                    while !sets_to_process.is_empty() {
                        let current_level_sets = std::mem::take(&mut sets_to_process);

                        for set_id in current_level_sets {
                            let members = self
                                .cala
                                .account_sets()
                                .list_members_by_created_at(set_id, Default::default())
                                .await?
                                .entities;

                            for member in members {
                                match member.id {
                                    cala_ledger::account_set::AccountSetMemberId::Account(
                                        account_id,
                                    ) => {
                                        cost_of_revenue_accounts.push((
                                            self.journal_id,
                                            account_id.clone(),
                                            Currency::USD,
                                        ));
                                    }
                                    cala_ledger::account_set::AccountSetMemberId::AccountSet(
                                        nested_set_id,
                                    ) => {
                                        sets_to_process.push(nested_set_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let revenue_account_balances = self.cala.balances().find_all(&revenue_accounts).await?;
        let cost_of_revenue_account_balances = self
            .cala
            .balances()
            .find_all(&cost_of_revenue_accounts)
            .await?;
        let expenses_account_balances = self.cala.balances().find_all(&expense_accounts).await?;
        
        // TODO: pass in db op?
        let op = self.repo.begin_op().await?.with_db_time().await?;
        let entries = self
            .chart_ledger
            .prepare_annual_closing_entries(
                op,
                revenue_account_balances,
                cost_of_revenue_account_balances,
                expenses_account_balances,
                retained_earnings_set_id,
                retained_losses_set_id,
            )
            .await?;

        Ok(entries)
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.add_root_node",
        skip(self,),
        err
    )]
    pub async fn add_root_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartId> + std::fmt::Debug,
        spec: impl Into<AccountSpec> + std::fmt::Debug,
    ) -> Result<(Chart, Option<CalaAccountSetId>), ChartOfAccountsError> {
        let id = id.into();
        let spec = spec.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;
        let mut chart = self.repo.find_by_id(id).await?;

        let es_entity::Idempotent::Executed(NewChartAccountDetails {
            parent_account_set_id,
            new_account_set,
        }) = chart.create_root_node(&spec, self.journal_id)
        else {
            return Ok((chart, None));
        };
        let account_set_id = new_account_set.id;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, parent_account_set_id, account_set_id)
            .await?;

        op.commit().await?;

        let new_account_set_id = chart.trial_balance_account_id_from_new_account(account_set_id);
        Ok((chart, new_account_set_id))
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.add_child_node",
        skip(self),
        err
    )]
    pub async fn add_child_node(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ChartId> + std::fmt::Debug,
        parent_code: AccountCode,
        code: AccountCode,
        name: AccountName,
    ) -> Result<(Chart, Option<CalaAccountSetId>), ChartOfAccountsError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::chart(id),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;
        let mut chart = self.repo.find_by_id(id).await?;

        let es_entity::Idempotent::Executed(NewChartAccountDetails {
            parent_account_set_id,
            new_account_set,
        }) = chart.create_child_node(parent_code, code, name, self.journal_id)?
        else {
            return Ok((chart, None));
        };
        let account_set_id = new_account_set.id;

        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, parent_account_set_id, account_set_id)
            .await?;

        op.commit().await?;

        let new_account_set_id = chart.trial_balance_account_id_from_new_account(account_set_id);
        Ok((chart, new_account_set_id))
    }

    #[instrument(name = "core_accounting.chart_of_accounts.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        id: impl Into<ChartId> + std::fmt::Debug,
    ) -> Result<Chart, ChartOfAccountsError> {
        self.repo.find_by_id(id.into()).await
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.find_by_reference_with_sub",
        skip(self),
        err
    )]
    pub async fn find_by_reference_with_sub(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        reference: &str,
    ) -> Result<Option<Chart>, ChartOfAccountsError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_LIST,
            )
            .await?;

        self.find_by_reference(reference).await
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.find_by_reference",
        skip(self),
        err
    )]
    pub async fn find_by_reference(
        &self,
        reference: &str,
    ) -> Result<Option<Chart>, ChartOfAccountsError> {
        let reference = reference.to_string();
        let chart = match self.repo.find_by_reference(reference).await {
            Ok(chart) => Some(chart),
            Err(e) if e.was_not_found() => None,
            Err(e) => return Err(e),
        };

        Ok(chart)
    }

    #[instrument(name = "core_accounting.chart_of_accounts.find_all", skip(self), err)]
    pub async fn find_all<T: From<Chart>>(
        &self,
        ids: &[ChartId],
    ) -> Result<std::collections::HashMap<ChartId, T>, ChartOfAccountsError> {
        self.repo.find_all(ids).await
    }

    #[instrument(
        name = "core_accounting.chart_of_accounts.manual_transaction_account_id_for_account_id_or_code",
        skip(self),
        err
    )]
    pub async fn manual_transaction_account_id_for_account_id_or_code(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        account_id_or_code: AccountIdOrCode,
    ) -> Result<LedgerAccountId, ChartOfAccountsError> {
        let mut chart = self.repo.find_by_reference(chart_ref.to_string()).await?;

        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_charts(),
                CoreAccountingAction::CHART_UPDATE,
            )
            .await?;

        let manual_transaction_account_id = match chart
            .manual_transaction_account(account_id_or_code)?
        {
            ManualAccountFromChart::IdInChart(id) | ManualAccountFromChart::NonChartId(id) => id,
            ManualAccountFromChart::NewAccount((account_set_id, new_account)) => {
                let mut op = self.repo.begin_op().await?;
                self.repo.update_in_op(&mut op, &mut chart).await?;

                let mut op = self
                    .cala
                    .ledger_operation_from_db_op(op.with_db_time().await?);
                let Account {
                    id: manual_transaction_account_id,
                    ..
                } = self
                    .cala
                    .accounts()
                    .create_in_op(&mut op, new_account)
                    .await?;

                self.cala
                    .account_sets()
                    .add_member_in_op(&mut op, account_set_id, manual_transaction_account_id)
                    .await?;

                op.commit().await?;

                manual_transaction_account_id.into()
            }
        };

        Ok(manual_transaction_account_id)
    }
}
