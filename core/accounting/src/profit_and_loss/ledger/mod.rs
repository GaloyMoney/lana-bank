pub mod error;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;

use cala_ledger::{
    AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId,
    account_set::{AccountSet, AccountSetMemberId, AccountSetUpdate, NewAccountSet},
};

use audit::AuditInfo;
use tracing_macros::record_error_severity;

use crate::primitives::{BalanceRange, CalaBalanceRange};

use super::{
    COST_OF_REVENUE_NAME, ChartOfAccountsIntegrationConfig, EXPENSES_NAME, ProfitAndLossStatement,
    ProfitAndLossStatementIds, REVENUE_NAME,
};

use error::*;

type AccountSetWithBalances = (
    AccountSet,
    (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
);

#[derive(Clone)]
pub struct ProfitAndLossStatementLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl ProfitAndLossStatementLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.create_unique_account_set", skip(self, op, parents), fields(reference = %reference, normal_balance_type = ?normal_balance_type, parents_count = parents.len()))]
    async fn create_unique_account_set(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reference: &str,
        normal_balance_type: DebitOrCredit,
        parents: Vec<AccountSetId>,
    ) -> Result<AccountSetId, ProfitAndLossStatementLedgerError> {
        let id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(id)
            .journal_id(self.journal_id)
            .external_id(reference)
            .name(reference)
            .description(reference)
            .normal_balance_type(normal_balance_type)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(op, new_account_set)
            .await?;

        for parent_id in parents {
            self.cala
                .account_sets()
                .add_member_in_op(op, parent_id, id)
                .await?;
        }

        Ok(id)
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.create_account_set", skip(self, op, parents), fields(reference = %reference, normal_balance_type = ?normal_balance_type, parents_count = parents.len()))]
    async fn create_account_set(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reference: &str,
        normal_balance_type: DebitOrCredit,
        parents: Vec<AccountSetId>,
    ) -> Result<AccountSetId, ProfitAndLossStatementLedgerError> {
        let id = AccountSetId::new();
        let new_account_set = NewAccountSet::builder()
            .id(id)
            .journal_id(self.journal_id)
            .name(reference)
            .description(reference)
            .normal_balance_type(normal_balance_type)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(op, new_account_set)
            .await?;

        for parent_id in parents {
            self.cala
                .account_sets()
                .add_member_in_op(op, parent_id, id)
                .await?;
        }

        Ok(id)
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.get_member_account_set_ids_and_names", skip_all)]
    async fn get_member_account_set_ids_and_names(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<HashMap<String, AccountSetId>, ProfitAndLossStatementLedgerError> {
        let id = id.into();

        let member_ids = self
            .cala
            .account_sets()
            .list_members_by_created_at(id, Default::default())
            .await?
            .entities
            .into_iter()
            .map(|m| match m.id {
                AccountSetMemberId::AccountSet(id) => Ok(id),
                _ => Err(ProfitAndLossStatementLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, ProfitAndLossStatementLedgerError>>()?;

        let mut accounts: HashMap<String, AccountSetId> = HashMap::new();
        for id in member_ids {
            let account_set = self.cala.account_sets().find(id).await?.into_values();
            accounts.insert(account_set.name, id);
        }

        Ok(accounts)
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.get_account_set_with_balances", skip(self, balances_by_id), fields(account_set_id = %account_set_id))]
    async fn get_account_set_with_balances(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &mut HashMap<BalanceId, CalaBalanceRange>,
    ) -> Result<AccountSetWithBalances, ProfitAndLossStatementLedgerError> {
        let account_set = self.cala.account_sets().find(account_set_id).await?;

        let btc_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::BTC));
        let usd_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::USD));

        Ok((account_set, (usd_balance, btc_balance)))
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.get_balances_by_id", skip(self, all_account_set_ids), fields(count = all_account_set_ids.len(), from = %from, until = ?until))]
    async fn get_balances_by_id(
        &self,
        all_account_set_ids: Vec<AccountSetId>,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<HashMap<BalanceId, CalaBalanceRange>, ProfitAndLossStatementLedgerError> {
        let balance_ids = all_account_set_ids
            .iter()
            .flat_map(|id| {
                [
                    (self.journal_id, (*id).into(), Currency::USD),
                    (self.journal_id, (*id).into(), Currency::BTC),
                ]
            })
            .collect::<Vec<_>>();
        let res = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&balance_ids, from, until)
            .await?;

        Ok(res)
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.attach_chart_of_accounts_account_sets", skip(self, charts_integration_meta), fields(reference = %reference))]
    pub async fn attach_chart_of_accounts_account_sets(
        &self,
        reference: String,
        charts_integration_meta: ChartOfAccountsIntegrationMeta,
    ) -> Result<(), ProfitAndLossStatementLedgerError> {
        let mut op = self.cala.begin_operation().await?;

        let account_set_ids = self.get_ids_from_reference(reference).await?;
        let mut account_sets = self
            .cala
            .account_sets()
            .find_all_in_op::<AccountSet>(&mut op, &account_set_ids.internal_ids())
            .await?;

        let ChartOfAccountsIntegrationMeta {
            config: _,
            audit_info: _,

            revenue_child_account_set_id_from_chart,
            cost_of_revenue_child_account_set_id_from_chart,
            expenses_child_account_set_id_from_chart,
        } = &charts_integration_meta;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            account_set_ids.revenue,
            *revenue_child_account_set_id_from_chart,
            &charts_integration_meta,
            |meta| meta.revenue_child_account_set_id_from_chart,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            account_set_ids.cost_of_revenue,
            *cost_of_revenue_child_account_set_id_from_chart,
            &charts_integration_meta,
            |meta| meta.cost_of_revenue_child_account_set_id_from_chart,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            account_set_ids.expenses,
            *expenses_child_account_set_id_from_chart,
            &charts_integration_meta,
            |meta| meta.expenses_child_account_set_id_from_chart,
        )
        .await?;

        op.commit().await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "profit_and_loss.attach_charts_account_set",
        skip(self, op, account_sets, new_meta, old_parent_id_getter)
    )]
    async fn attach_charts_account_set<F>(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        account_sets: &mut HashMap<AccountSetId, AccountSet>,
        internal_account_set_id: AccountSetId,
        child_account_set_id_from_chart: AccountSetId,
        new_meta: &ChartOfAccountsIntegrationMeta,
        old_parent_id_getter: F,
    ) -> Result<(), ProfitAndLossStatementLedgerError>
    where
        F: FnOnce(ChartOfAccountsIntegrationMeta) -> AccountSetId,
    {
        let mut internal_account_set = account_sets
            .remove(&internal_account_set_id)
            .expect("internal account set not found");

        if let Some(old_meta) = internal_account_set.values().metadata.as_ref() {
            let old_meta: ChartOfAccountsIntegrationMeta =
                serde_json::from_value(old_meta.clone()).expect("Could not deserialize metadata");
            let old_child_account_set_id_from_chart = old_parent_id_getter(old_meta);
            if old_child_account_set_id_from_chart != child_account_set_id_from_chart {
                self.cala
                    .account_sets()
                    .remove_member_in_op(
                        op,
                        internal_account_set_id,
                        old_child_account_set_id_from_chart,
                    )
                    .await?;
            }
        }

        self.cala
            .account_sets()
            .add_member_in_op(op, internal_account_set_id, child_account_set_id_from_chart)
            .await?;
        let mut update = AccountSetUpdate::default();
        update
            .metadata(new_meta)
            .expect("Could not update metadata");
        if internal_account_set.update(update).did_execute() {
            self.cala
                .account_sets()
                .persist_in_op(op, &mut internal_account_set)
                .await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.create", skip(self, op), fields(reference = %reference))]
    pub async fn create(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<ProfitAndLossStatementIds, ProfitAndLossStatementLedgerError> {
        let statement_id = self
            .create_unique_account_set(op, reference, DebitOrCredit::Credit, vec![])
            .await?;

        let revenue_id = self
            .create_account_set(op, REVENUE_NAME, DebitOrCredit::Credit, vec![statement_id])
            .await?;
        let expenses_id = self
            .create_account_set(op, EXPENSES_NAME, DebitOrCredit::Debit, vec![statement_id])
            .await?;

        let cost_of_revenue_id = self
            .create_account_set(
                op,
                COST_OF_REVENUE_NAME,
                DebitOrCredit::Debit,
                vec![statement_id],
            )
            .await?;
        Ok(ProfitAndLossStatementIds {
            id: statement_id,
            revenue: revenue_id,
            expenses: expenses_id,
            cost_of_revenue: cost_of_revenue_id,
        })
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.get_pl_statement", skip(self), fields(reference = %reference, from = %from, until = ?until))]
    pub async fn get_pl_statement(
        &self,
        reference: String,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<ProfitAndLossStatement, ProfitAndLossStatementLedgerError> {
        let ids = self.get_ids_from_reference(reference).await?;
        let all_account_set_ids = vec![ids.id, ids.revenue, ids.expenses];

        let mut balances_by_id = self
            .get_balances_by_id(all_account_set_ids, from, until)
            .await?;

        let (account, balances) = self
            .get_account_set_with_balances(ids.id, &mut balances_by_id)
            .await?;

        Ok(ProfitAndLossStatement::from((account, balances, ids)))
    }

    #[record_error_severity]
    #[instrument(name = "pl_ledger.get_chart_of_accounts_integration_config", skip(self), fields(reference = %reference))]
    pub async fn get_chart_of_accounts_integration_config(
        &self,
        reference: String,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, ProfitAndLossStatementLedgerError> {
        let account_set_id = self
            .get_ids_from_reference(reference)
            .await?
            .account_set_id_for_config();

        let account_set = self.cala.account_sets().find(account_set_id).await?;
        if let Some(meta) = account_set.values().metadata.as_ref() {
            let meta: ChartOfAccountsIntegrationMeta =
                serde_json::from_value(meta.clone()).expect("Could not deserialize metadata");
            Ok(Some(meta.config))
        } else {
            Ok(None)
        }
    }

    #[record_error_severity]
    #[instrument(name = "profit_and_loss.get_ids_from_reference", skip(self), fields(reference = %reference))]
    pub async fn get_ids_from_reference(
        &self,
        reference: String,
    ) -> Result<ProfitAndLossStatementIds, ProfitAndLossStatementLedgerError> {
        let statement_id = self
            .cala
            .account_sets()
            .find_by_external_id(reference)
            .await?
            .id;

        let statement_members = self
            .get_member_account_set_ids_and_names(statement_id)
            .await?;

        let expenses_id = statement_members.get(EXPENSES_NAME).ok_or(
            ProfitAndLossStatementLedgerError::NotFound(EXPENSES_NAME.to_string()),
        )?;

        let revenue_id = statement_members.get(REVENUE_NAME).ok_or(
            ProfitAndLossStatementLedgerError::NotFound(REVENUE_NAME.to_string()),
        )?;

        let cost_of_revenue_id = statement_members.get(COST_OF_REVENUE_NAME).ok_or(
            ProfitAndLossStatementLedgerError::NotFound(COST_OF_REVENUE_NAME.to_string()),
        )?;

        Ok(ProfitAndLossStatementIds {
            id: statement_id,
            revenue: *revenue_id,
            cost_of_revenue: *cost_of_revenue_id,
            expenses: *expenses_id,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChartOfAccountsIntegrationMeta {
    pub config: ChartOfAccountsIntegrationConfig,
    pub audit_info: AuditInfo,

    pub revenue_child_account_set_id_from_chart: AccountSetId,
    pub cost_of_revenue_child_account_set_id_from_chart: AccountSetId,
    pub expenses_child_account_set_id_from_chart: AccountSetId,
}

impl
    From<(
        AccountSet,
        (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
        ProfitAndLossStatementIds,
    )> for ProfitAndLossStatement
{
    fn from(
        (account_set, (usd_balance, btc_balance), ids): (
            AccountSet,
            (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
            ProfitAndLossStatementIds,
        ),
    ) -> Self {
        let values = account_set.into_values();

        let usd_balance_range = usd_balance.map(|range| BalanceRange {
            open: Some(range.open),
            close: Some(range.close),
            period_activity: Some(range.period),
        });

        let btc_balance_range = btc_balance.map(|range| BalanceRange {
            open: Some(range.open),
            close: Some(range.close),
            period_activity: Some(range.period),
        });

        ProfitAndLossStatement {
            id: values.id.into(),
            name: values.name,
            usd_balance_range,
            btc_balance_range,
            category_ids: vec![
                ids.revenue.into(),
                ids.cost_of_revenue.into(),
                ids.expenses.into(),
            ],
        }
    }
}
