pub mod error;

use chrono::NaiveDate;

use cala_ledger::{
    AccountSetId, BalanceId, CalaLedger, Currency, DebitOrCredit, JournalId, LedgerOperation,
    account::{Account, AccountId},
    account_set::{AccountSet, NewAccountSet},
};

use crate::primitives::{AccountCode, BalanceRange, CalaBalanceRange, LedgerAccountId};

use super::TrialBalanceRow;

use error::*;

#[derive(Clone)]
pub struct TrialBalanceRoot {
    pub id: AccountSetId,
    pub name: String,
    pub description: Option<String>,
    pub usd_balance_range: Option<BalanceRange>,
    pub btc_balance_range: Option<BalanceRange>,
    pub from: NaiveDate,
    pub until: Option<NaiveDate>,
}

#[derive(Clone)]
pub struct TrialBalanceLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl TrialBalanceLedger {
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
    ) -> Result<AccountSetId, TrialBalanceLedgerError> {
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

    async fn get_account_set_with_balances(
        &self,
        account_set_id: AccountSetId,
        balances_by_id: &mut std::collections::HashMap<BalanceId, CalaBalanceRange>,
    ) -> Result<
        (
            AccountSet,
            (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
        ),
        TrialBalanceLedgerError,
    > {
        let account_set = self.cala.account_sets().find(account_set_id).await?;

        let btc_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::BTC));
        let usd_balance =
            balances_by_id.remove(&(self.journal_id, account_set_id.into(), Currency::USD));

        Ok((account_set, (btc_balance, usd_balance)))
    }

    async fn get_balances_by_id(
        &self,
        all_account_set_ids: Vec<AccountSetId>,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<std::collections::HashMap<BalanceId, CalaBalanceRange>, TrialBalanceLedgerError>
    {
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
    ) -> Result<(), TrialBalanceLedgerError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        self.add_member_in_op(&mut op, node_account_set_id, member)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn add_members(
        &self,
        op: es_entity::DbOp<'_>,
        node_account_set_id: impl Into<AccountSetId> + Copy,
        members: impl Iterator<Item = &AccountSetId>,
    ) -> Result<(), TrialBalanceLedgerError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);
        for member in members {
            self.add_member_in_op(&mut op, node_account_set_id, *member)
                .await?;
        }

        op.commit().await?;
        Ok(())
    }

    async fn add_member_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        node_account_set_id: impl Into<AccountSetId>,
        member: AccountSetId,
    ) -> Result<(), TrialBalanceLedgerError> {
        let node_account_set_id = node_account_set_id.into();

        self.cala
            .account_sets()
            .add_member_in_op(op, node_account_set_id, member)
            .await?;

        Ok(())
    }

    pub async fn create(
        &self,
        op: es_entity::DbOp<'_>,
        reference: &str,
    ) -> Result<AccountSetId, TrialBalanceLedgerError> {
        let mut op = self
            .cala
            .ledger_operation_from_db_op(op.with_db_time().await?);

        let statement_id = self
            .create_unique_account_set(&mut op, reference, DebitOrCredit::Debit, vec![])
            .await?;

        op.commit().await?;
        Ok(statement_id)
    }

    pub async fn get_id_from_reference(
        &self,
        reference: String,
    ) -> Result<AccountSetId, TrialBalanceLedgerError> {
        Ok(self
            .cala
            .account_sets()
            .find_by_external_id(reference)
            .await?
            .id)
    }

    pub async fn get_trial_balance(
        &self,
        name: String,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<TrialBalanceRoot, TrialBalanceLedgerError> {
        let statement_id = self.get_id_from_reference(name).await?;

        let mut balances_by_id = self
            .get_balances_by_id(vec![statement_id], from, until)
            .await?;

        let (account, balances) = self
            .get_account_set_with_balances(statement_id, &mut balances_by_id)
            .await?;

        Ok(TrialBalanceRoot::from((account, balances, from, until)))
    }

    pub async fn load_accounts_in_range(
        &self,
        ids: &[LedgerAccountId],
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<Vec<TrialBalanceRow>, TrialBalanceLedgerError> {
        let account_set_ids: Vec<AccountSetId> = ids.iter().map(|id| (*id).into()).collect();
        let account_ids: Vec<AccountId> = ids.iter().map(|id| (*id).into()).collect();
        let balance_ids = ids
            .iter()
            .flat_map(|id| {
                [
                    (self.journal_id, (*id).into(), Currency::USD),
                    (self.journal_id, (*id).into(), Currency::BTC),
                ]
            })
            .collect::<Vec<_>>();

        let (account_sets_result, accounts_result, balances_result) = tokio::join!(
            self.cala
                .account_sets()
                .find_all::<AccountSet>(&account_set_ids),
            self.cala.accounts().find_all::<Account>(&account_ids),
            self.cala
                .balances()
                .effective()
                .find_all_in_range(&balance_ids, from, until)
        );

        let mut account_sets = account_sets_result?;
        let mut accounts = accounts_result?;
        let mut balances = balances_result?;

        let mut rows = Vec::with_capacity(ids.len());
        for ledger_id in ids {
            let ledger_id = *ledger_id;
            let account_set_id: AccountSetId = ledger_id.into();

            if let Some(account_set) = account_sets.remove(&account_set_id) {
                let btc_balance =
                    balances.remove(&(self.journal_id, ledger_id.into(), Currency::BTC));
                let usd_balance =
                    balances.remove(&(self.journal_id, ledger_id.into(), Currency::USD));

                let row = TrialBalanceRow::from((account_set, (btc_balance, usd_balance)));
                if row.has_non_zero_activity() {
                    rows.push(row);
                }
                continue;
            }

            let account_id: AccountId = ledger_id.into();
            if let Some(account) = accounts.remove(&account_id) {
                let btc_balance =
                    balances.remove(&(self.journal_id, ledger_id.into(), Currency::BTC));
                let usd_balance =
                    balances.remove(&(self.journal_id, ledger_id.into(), Currency::USD));

                let row = TrialBalanceRow::from((account, (btc_balance, usd_balance)));
                if row.has_non_zero_activity() {
                    rows.push(row);
                }
            }
        }

        Ok(rows)
    }
}

impl
    From<(
        AccountSet,
        (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
        NaiveDate,
        Option<NaiveDate>,
    )> for TrialBalanceRoot
{
    fn from(
        (account_set, (btc_balance, usd_balance), from, until): (
            AccountSet,
            (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
            NaiveDate,
            Option<NaiveDate>,
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
        TrialBalanceRoot {
            id: values.id,
            name: values.name,
            description: values.description,
            btc_balance_range,
            usd_balance_range,
            from,
            until,
        }
    }
}

impl
    From<(
        AccountSet,
        (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
    )> for TrialBalanceRow
{
    fn from(
        (account_set, (btc_balance, usd_balance)): (
            AccountSet,
            (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
        ),
    ) -> Self {
        let values = account_set.into_values();
        let code = values
            .external_id
            .as_ref()
            .and_then(|id| id.parse::<AccountCode>().ok());

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

        TrialBalanceRow {
            id: values.id.into(),
            name: values.name,
            code,
            normal_balance_type: values.normal_balance_type,
            usd_balance_range,
            btc_balance_range,
        }
    }
}

impl
    From<(
        Account,
        (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
    )> for TrialBalanceRow
{
    fn from(
        (account, (btc_balance, usd_balance)): (
            Account,
            (Option<CalaBalanceRange>, Option<CalaBalanceRange>),
        ),
    ) -> Self {
        let values = account.into_values();
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

        TrialBalanceRow {
            id: values.id.into(),
            name: values.name,
            code: None,
            normal_balance_type: values.normal_balance_type,
            usd_balance_range,
            btc_balance_range,
        }
    }
}
