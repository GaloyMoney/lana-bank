pub mod error;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use cala_ledger::{
    AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
    account_set::{AccountSet, AccountSetMemberId, AccountSetUpdate, NewAccountSet},
};

use audit::AuditInfo;

use crate::primitives::{BalanceRange, CalaBalanceRange};

use super::{
    ASSETS_NAME, BalanceSheet, BalanceSheetIds, COST_OF_REVENUE_NAME,
    ChartOfAccountsIntegrationConfig, EQUITY_NAME, EXPENSES_NAME, LIABILITIES_NAME,
    NET_INCOME_NAME, REVENUE_NAME,
};

use error::*;

#[derive(Clone)]
pub struct BalanceSheetLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl BalanceSheetLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    async fn create_unique_account_set(
        &self,
        op: &mut LedgerOperation<'_>,
        reference: &str,
        normal_balance_type: DebitOrCredit,
        parents: Vec<AccountSetId>,
    ) -> Result<AccountSetId, BalanceSheetLedgerError> {
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

    async fn create_account_set(
        &self,
        op: &mut LedgerOperation<'_>,
        reference: &str,
        normal_balance_type: DebitOrCredit,
        parents: Vec<AccountSetId>,
    ) -> Result<AccountSetId, BalanceSheetLedgerError> {
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

    async fn get_member_account_set_ids_and_names(
        &self,
        id: impl Into<AccountSetId> + Copy,
    ) -> Result<HashMap<String, AccountSetId>, BalanceSheetLedgerError> {
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
                _ => Err(BalanceSheetLedgerError::NonAccountSetMemberTypeFound),
            })
            .collect::<Result<Vec<AccountSetId>, BalanceSheetLedgerError>>()?;

        let mut accounts: HashMap<String, AccountSetId> = HashMap::new();
        for id in member_ids {
            let account_set = self.cala.account_sets().find(id).await?.into_values();
            accounts.insert(account_set.name, id);
        }

        Ok(accounts)
    }

    async fn get_account_set_with_balances(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &mut HashMap<BalanceId, CalaBalanceRange>,
    ) -> Result<
        (
            AccountSet,
            (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
        ),
        BalanceSheetLedgerError,
    > {
        let account_set = self.cala.account_sets().find(account_set_id).await?;

        let btc_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::BTC));
        let usd_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::USD));

        Ok((account_set, (usd_balance, btc_balance)))
    }

    async fn get_balances_by_id(
        &self,
        all_account_set_ids: Vec<AccountSetId>,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<HashMap<BalanceId, CalaBalanceRange>, BalanceSheetLedgerError> {
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

    pub async fn add_member(
        &self,
        op: es_entity::DbOp<'_>,
        node_account_set_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), BalanceSheetLedgerError> {
        let node_account_set_id = node_account_set_id.into();

        let mut op = self.cala.ledger_operation_from_db_op(op);
        match self
            .cala
            .account_sets()
            .add_member_in_op(&mut op, node_account_set_id, member)
            .await
        {
            Ok(_) | Err(cala_ledger::account_set::error::AccountSetError::MemberAlreadyAdded) => {}
            Err(e) => return Err(e.into()),
        }

        op.commit().await?;
        Ok(())
    }

    pub async fn create(
        &self,
        op: es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<BalanceSheetIds, BalanceSheetLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let statement_id = self
            .create_unique_account_set(&mut op, reference, DebitOrCredit::Debit, vec![])
            .await?;

        let assets_id = self
            .create_account_set(
                &mut op,
                ASSETS_NAME,
                DebitOrCredit::Debit,
                vec![statement_id],
            )
            .await?;
        let liabilities_id = self
            .create_account_set(
                &mut op,
                LIABILITIES_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;
        let equity_id = self
            .create_account_set(
                &mut op,
                EQUITY_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;

        let net_income_id = self
            .create_account_set(
                &mut op,
                NET_INCOME_NAME,
                DebitOrCredit::Credit,
                vec![equity_id],
            )
            .await?;

        let revenue_id = self
            .create_account_set(
                &mut op,
                REVENUE_NAME,
                DebitOrCredit::Credit,
                vec![net_income_id],
            )
            .await?;
        let cost_of_revenue_id = self
            .create_account_set(
                &mut op,
                COST_OF_REVENUE_NAME,
                DebitOrCredit::Debit,
                vec![net_income_id],
            )
            .await?;
        let expenses_id = self
            .create_account_set(
                &mut op,
                EXPENSES_NAME,
                DebitOrCredit::Debit,
                vec![net_income_id],
            )
            .await?;

        op.commit().await?;

        Ok(BalanceSheetIds {
            id: statement_id,
            assets: assets_id,
            liabilities: liabilities_id,
            equity: equity_id,
            revenue: revenue_id,
            cost_of_revenue: cost_of_revenue_id,
            expenses: expenses_id,
        })
    }

    pub async fn get_ids_from_reference(
        &self,
        reference: String,
    ) -> Result<BalanceSheetIds, BalanceSheetLedgerError> {
        let statement_id = self
            .cala
            .account_sets()
            .find_by_external_id(reference)
            .await?
            .id;

        let statement_members = self
            .get_member_account_set_ids_and_names(statement_id)
            .await?;
        let assets_id = statement_members
            .get(ASSETS_NAME)
            .ok_or(BalanceSheetLedgerError::NotFound(ASSETS_NAME.to_string()))?;
        let liabilities_id =
            statement_members
                .get(LIABILITIES_NAME)
                .ok_or(BalanceSheetLedgerError::NotFound(
                    LIABILITIES_NAME.to_string(),
                ))?;
        let equity_id = statement_members
            .get(EQUITY_NAME)
            .ok_or(BalanceSheetLedgerError::NotFound(EQUITY_NAME.to_string()))?;

        let equity_members = self
            .get_member_account_set_ids_and_names(*equity_id)
            .await?;
        let net_income_id =
            equity_members
                .get(NET_INCOME_NAME)
                .ok_or(BalanceSheetLedgerError::NotFound(
                    NET_INCOME_NAME.to_string(),
                ))?;

        let net_income_members = self
            .get_member_account_set_ids_and_names(*net_income_id)
            .await?;
        let revenue_id = net_income_members
            .get(REVENUE_NAME)
            .ok_or(BalanceSheetLedgerError::NotFound(REVENUE_NAME.to_string()))?;
        let cost_of_revenue_id = net_income_members.get(COST_OF_REVENUE_NAME).ok_or(
            BalanceSheetLedgerError::NotFound(COST_OF_REVENUE_NAME.to_string()),
        )?;
        let expenses_id = net_income_members
            .get(EXPENSES_NAME)
            .ok_or(BalanceSheetLedgerError::NotFound(EXPENSES_NAME.to_string()))?;

        Ok(BalanceSheetIds {
            id: statement_id,
            assets: *assets_id,
            liabilities: *liabilities_id,
            equity: *equity_id,
            revenue: *revenue_id,
            cost_of_revenue: *cost_of_revenue_id,
            expenses: *expenses_id,
        })
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        reference: String,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, BalanceSheetLedgerError> {
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

    async fn attach_charts_account_set<F>(
        &self,
        op: &mut LedgerOperation<'_>,
        account_sets: &mut HashMap<AccountSetId, AccountSet>,
        internal_account_set_id: AccountSetId,
        child_account_set_id_from_chart: AccountSetId,
        new_meta: &ChartOfAccountsIntegrationMeta,
        old_parent_id_getter: F,
    ) -> Result<(), BalanceSheetLedgerError>
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
        internal_account_set.update(update);
        self.cala
            .account_sets()
            .persist_in_op(op, &mut internal_account_set)
            .await?;

        Ok(())
    }

    pub async fn attach_chart_of_accounts_account_sets(
        &self,
        reference: String,
        charts_integration_meta: ChartOfAccountsIntegrationMeta,
    ) -> Result<(), BalanceSheetLedgerError> {
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

            assets_child_account_set_id_from_chart,
            liabilities_child_account_set_id_from_chart,
            equity_child_account_set_id_from_chart,
            revenue_child_account_set_id_from_chart,
            cost_of_revenue_child_account_set_id_from_chart,
            expenses_child_account_set_id_from_chart,
        } = &charts_integration_meta;

        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            account_set_ids.assets,
            *assets_child_account_set_id_from_chart,
            &charts_integration_meta,
            |meta| meta.assets_child_account_set_id_from_chart,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            account_set_ids.liabilities,
            *liabilities_child_account_set_id_from_chart,
            &charts_integration_meta,
            |meta| meta.liabilities_child_account_set_id_from_chart,
        )
        .await?;
        self.attach_charts_account_set(
            &mut op,
            &mut account_sets,
            account_set_ids.equity,
            *equity_child_account_set_id_from_chart,
            &charts_integration_meta,
            |meta| meta.equity_child_account_set_id_from_chart,
        )
        .await?;
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

    pub async fn get_balance_sheet(
        &self,
        reference: String,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<BalanceSheet, BalanceSheetLedgerError> {
        let ids = self.get_ids_from_reference(reference).await?;
        let all_account_set_ids = vec![ids.id, ids.assets, ids.liabilities, ids.equity];

        let mut balances_by_id = self
            .get_balances_by_id(all_account_set_ids, from, until)
            .await?;

        let (account, balances) = self
            .get_account_set_with_balances(ids.id, &mut balances_by_id)
            .await?;

        Ok(BalanceSheet::from((account, balances, ids)))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChartOfAccountsIntegrationMeta {
    pub config: ChartOfAccountsIntegrationConfig,
    pub audit_info: AuditInfo,

    pub assets_child_account_set_id_from_chart: AccountSetId,
    pub liabilities_child_account_set_id_from_chart: AccountSetId,
    pub equity_child_account_set_id_from_chart: AccountSetId,
    pub revenue_child_account_set_id_from_chart: AccountSetId,
    pub cost_of_revenue_child_account_set_id_from_chart: AccountSetId,
    pub expenses_child_account_set_id_from_chart: AccountSetId,
}

impl
    From<(
        AccountSet,
        (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
        BalanceSheetIds,
    )> for BalanceSheet
{
    fn from(
        (account_set, (usd_balance, btc_balance), ids): (
            AccountSet,
            (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
            BalanceSheetIds,
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

        BalanceSheet {
            id: values.id.into(),
            name: values.name,
            usd_balance_range,
            btc_balance_range,
            category_ids: vec![ids.equity.into(), ids.assets.into(), ids.liabilities.into()],
        }
    }
}
