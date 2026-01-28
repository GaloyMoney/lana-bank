pub mod error;

use chrono::NaiveDate;
use std::collections::HashMap;
use tracing::instrument;

use cala_ledger::{
    AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId,
    account_set::{AccountSet, AccountSetMemberId, NewAccountSet},
};
use tracing_macros::record_error_severity;

use crate::primitives::{BalanceRange, CalaBalanceRange, ResolvedAccountingBaseConfig};

use super::{
    ASSETS_NAME, BalanceSheet, BalanceSheetIds, COST_OF_REVENUE_NAME, EQUITY_NAME, EXPENSES_NAME,
    LIABILITIES_NAME, NET_INCOME_NAME, REVENUE_NAME,
};

use error::*;

type AccountSetWithBalances = (
    AccountSet,
    (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
);

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

    #[record_error_severity]
    #[instrument(name = "bs_ledger.create_unique_account_set", skip(self, op, parents), fields(reference = %reference, normal_balance_type = ?normal_balance_type, parents_count = parents.len()))]
    async fn create_unique_account_set(
        &self,
        op: &mut es_entity::DbOp<'_>,
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

    #[record_error_severity]
    #[instrument(name = "bs_ledger.create_account_set", skip(self, op, parents), fields(reference = %reference, normal_balance_type = ?normal_balance_type, parents_count = parents.len()))]
    async fn create_account_set(
        &self,
        op: &mut es_entity::DbOp<'_>,
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

    #[record_error_severity]
    #[instrument(name = "bs_ledger.get_member_account_set_ids_and_names", skip_all)]
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

    #[record_error_severity]
    #[instrument(name = "bs_ledger.get_account_set_with_balances", skip(self, balances_by_id), fields(account_set_id = %account_set_id))]
    async fn get_account_set_with_balances(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &mut HashMap<BalanceId, CalaBalanceRange>,
    ) -> Result<AccountSetWithBalances, BalanceSheetLedgerError> {
        let account_set = self.cala.account_sets().find(account_set_id).await?;

        let btc_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::BTC));
        let usd_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::USD));

        Ok((account_set, (usd_balance, btc_balance)))
    }

    #[record_error_severity]
    #[instrument(name = "bs_ledger.get_balances_by_id", skip(self, all_account_set_ids), fields(count = all_account_set_ids.len(), from = %from, until = ?until))]
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

    #[record_error_severity]
    #[instrument(name = "bs_ledger.create", skip(self, op), fields(reference = %reference))]
    pub async fn create(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<BalanceSheetIds, BalanceSheetLedgerError> {
        let statement_id = self
            .create_unique_account_set(op, reference, DebitOrCredit::Debit, vec![])
            .await?;

        let assets_id = self
            .create_account_set(op, ASSETS_NAME, DebitOrCredit::Debit, vec![statement_id])
            .await?;
        let liabilities_id = self
            .create_account_set(
                op,
                LIABILITIES_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;
        let equity_id = self
            .create_account_set(op, EQUITY_NAME, DebitOrCredit::Credit, vec![statement_id])
            .await?;

        let net_income_id = self
            .create_account_set(op, NET_INCOME_NAME, DebitOrCredit::Credit, vec![equity_id])
            .await?;

        let revenue_id = self
            .create_account_set(op, REVENUE_NAME, DebitOrCredit::Credit, vec![net_income_id])
            .await?;
        let cost_of_revenue_id = self
            .create_account_set(
                op,
                COST_OF_REVENUE_NAME,
                DebitOrCredit::Debit,
                vec![net_income_id],
            )
            .await?;
        let expenses_id = self
            .create_account_set(op, EXPENSES_NAME, DebitOrCredit::Debit, vec![net_income_id])
            .await?;
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

    #[record_error_severity]
    #[instrument(name = "balance_sheet.get_ids_from_reference", skip(self), fields(reference = %reference))]
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

    pub(crate) async fn attach_chart_of_accounts_account_sets_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        reference: String,
        resolved: &ResolvedAccountingBaseConfig,
    ) -> Result<(), BalanceSheetLedgerError> {
        let account_set_ids = self.get_ids_from_reference(reference).await?;

        let pairs = [
            (account_set_ids.assets, resolved.assets),
            (account_set_ids.liabilities, resolved.liabilities),
            (account_set_ids.equity, resolved.equity),
            (account_set_ids.revenue, resolved.revenue),
            (account_set_ids.cost_of_revenue, resolved.cost_of_revenue),
            (account_set_ids.expenses, resolved.expenses),
        ];

        for (parent, child) in pairs {
            let members = self
                .cala
                .account_sets()
                .list_members_by_created_at(parent, Default::default())
                .await?
                .entities;

            let already_linked = members
                .iter()
                .any(|m| matches!(&m.id, AccountSetMemberId::AccountSet(id) if *id == child));

            if !already_linked {
                self.cala
                    .account_sets()
                    .add_member_in_op(op, parent, child)
                    .await?;
            }
        }

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
