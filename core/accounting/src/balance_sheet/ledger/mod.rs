pub mod error;

use chrono::NaiveDate;
use std::collections::HashMap;
use tracing::instrument;

use cala_ledger::{
    AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId,
    account_set::{AccountSetMemberId, NewAccountSet},
};
use tracing_macros::record_error_severity;

use crate::primitives::{CalaAccountBalance, ResolvedAccountingBaseConfig};

use super::{
    ASSETS_NAME, AccountCategoryBalance, BalanceSheet, BalanceSheetIds, COST_OF_REVENUE_NAME,
    EQUITY_NAME, EXPENSES_NAME, LIABILITIES_NAME, NET_INCOME_NAME, REVENUE_NAME,
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

    #[record_error_severity]
    #[instrument(name = "bs_ledger.create_unique_account_set_in_op", skip(self, op, parents), fields(reference = %reference, normal_balance_type = ?normal_balance_type, parents_count = parents.len()))]
    async fn create_unique_account_set_in_op(
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
    #[instrument(name = "bs_ledger.create_account_set_in_op", skip(self, op, parents), fields(reference = %reference, normal_balance_type = ?normal_balance_type, parents_count = parents.len()))]
    async fn create_account_set_in_op(
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
    #[instrument(name = "bs_ledger.get_balances_by_id", skip(self, all_account_set_ids), fields(count = all_account_set_ids.len(), as_of = %as_of))]
    async fn get_balances_by_id(
        &self,
        all_account_set_ids: Vec<AccountSetId>,
        as_of: NaiveDate,
    ) -> Result<HashMap<BalanceId, CalaAccountBalance>, BalanceSheetLedgerError> {
        let balance_ids = all_account_set_ids
            .iter()
            .flat_map(|id| {
                [
                    (self.journal_id, (*id).into(), Currency::USD),
                    (self.journal_id, (*id).into(), Currency::BTC),
                ]
            })
            .collect::<Vec<_>>();
        let ranges = self
            .cala
            .balances()
            .effective()
            .find_all_in_range(&balance_ids, as_of, Some(as_of))
            .await?;

        Ok(ranges
            .into_iter()
            .map(|(id, range)| (id, range.close))
            .collect())
    }

    #[record_error_severity]
    #[instrument(name = "bs_ledger.create_in_op", skip(self, op), fields(reference = %reference))]
    pub async fn create_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<BalanceSheetIds, BalanceSheetLedgerError> {
        let statement_id = self
            .create_unique_account_set_in_op(op, reference, DebitOrCredit::Debit, vec![])
            .await?;

        let assets_id = self
            .create_account_set_in_op(op, ASSETS_NAME, DebitOrCredit::Debit, vec![statement_id])
            .await?;
        let liabilities_id = self
            .create_account_set_in_op(
                op,
                LIABILITIES_NAME,
                DebitOrCredit::Credit,
                vec![statement_id],
            )
            .await?;
        let equity_id = self
            .create_account_set_in_op(op, EQUITY_NAME, DebitOrCredit::Credit, vec![statement_id])
            .await?;

        let net_income_id = self
            .create_account_set_in_op(op, NET_INCOME_NAME, DebitOrCredit::Credit, vec![equity_id])
            .await?;

        let revenue_id = self
            .create_account_set_in_op(op, REVENUE_NAME, DebitOrCredit::Credit, vec![net_income_id])
            .await?;
        let cost_of_revenue_id = self
            .create_account_set_in_op(
                op,
                COST_OF_REVENUE_NAME,
                DebitOrCredit::Debit,
                vec![net_income_id],
            )
            .await?;
        let expenses_id = self
            .create_account_set_in_op(op, EXPENSES_NAME, DebitOrCredit::Debit, vec![net_income_id])
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
        op: &mut es_entity::DbOp<'_>,
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
        as_of: NaiveDate,
    ) -> Result<BalanceSheet, BalanceSheetLedgerError> {
        let ids = self.get_ids_from_reference(reference).await?;
        let all_account_set_ids = vec![ids.assets, ids.liabilities, ids.equity];

        let mut balances_by_id = self.get_balances_by_id(all_account_set_ids, as_of).await?;

        let account_set = self.cala.account_sets().find(ids.id).await?;
        let name = account_set.into_values().name;

        Ok(BalanceSheet {
            name,
            assets: self.get_account_set_balance(&mut balances_by_id, ids.assets),
            liabilities: self.get_account_set_balance(&mut balances_by_id, ids.liabilities),
            equity: self.get_account_set_balance(&mut balances_by_id, ids.equity),
            category_ids: vec![ids.assets.into(), ids.liabilities.into(), ids.equity.into()],
        })
    }

    fn get_account_set_balance(
        &self,
        balances_by_id: &mut HashMap<BalanceId, CalaAccountBalance>,
        account_set_id: AccountSetId,
    ) -> AccountCategoryBalance {
        AccountCategoryBalance {
            btc: balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::BTC)),
            usd: balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::USD)),
        }
    }
}
