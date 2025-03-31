pub mod error;

use cala_ledger::{
    account::Account, account_set::AccountSet, balance::AccountBalance, AccountSetId, CalaLedger,
    JournalId,
};

use std::collections::HashMap;

use super::{LedgerAccount, LedgerAccountId};

use error::*;

#[derive(Clone)]
pub struct LedgerAccountLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl LedgerAccountLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    pub async fn balance<T>(
        &self,
        account_set_id: AccountSetId,
    ) -> Result<T, LedgerAccountLedgerError>
    where
        T: From<Option<cala_ledger::balance::AccountBalance>>,
    {
        let usd = "USD".parse().unwrap();
        let btc = "BTC".parse().unwrap();
        let usd_key = (self.journal_id, account_set_id.into(), usd);
        let btc_key = (self.journal_id, account_set_id.into(), btc);
        let balance_ids = [usd_key, btc_key];
        let mut balances = self.cala.balances().find_all(&balance_ids).await?;
        let usd_balance = balances.remove(&usd_key);
        let btc_balance = balances.remove(&btc_key);
        let res = if let Some(usd_balance) = usd_balance {
            Some(usd_balance).into()
        } else if let Some(btc_balance) = btc_balance {
            Some(btc_balance).into()
        } else {
            None.into()
        };
        Ok(res)
    }

    pub async fn account_set_history<T, U>(
        &self,
        account_set_id: AccountSetId,
        cursor: es_entity::PaginatedQueryArgs<U>,
    ) -> Result<es_entity::PaginatedQueryRet<T, U>, LedgerAccountLedgerError>
    where
        T: TryFrom<cala_ledger::entry::Entry, Error = LedgerAccountLedgerError>,
        U: std::fmt::Debug + From<cala_ledger::entry::EntriesByCreatedAtCursor>,
        cala_ledger::entry::EntriesByCreatedAtCursor: From<U>,
    {
        let cala_cursor = es_entity::PaginatedQueryArgs {
            after: cursor
                .after
                .map(cala_ledger::entry::EntriesByCreatedAtCursor::from),
            first: cursor.first,
        };

        let ret = self
            .cala
            .entries()
            .list_for_account_set_id(
                account_set_id,
                cala_cursor,
                es_entity::ListDirection::Descending,
            )
            .await?;

        let entities = ret
            .entities
            .into_iter()
            .map(T::try_from)
            .collect::<Result<Vec<T>, _>>()?;

        Ok(es_entity::PaginatedQueryRet {
            entities,
            has_next_page: ret.has_next_page,
            end_cursor: ret.end_cursor.map(U::from),
        })
    }

    pub async fn load_ledger_accounts<T: From<LedgerAccount>>(
        &self,
        ids: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, T>, LedgerAccountLedgerError> {
        let account_set_ids = ids.iter().map(|id| (*id).into()).collect::<Vec<_>>();
        let account_ids = ids.iter().map(|id| (*id).into()).collect::<Vec<_>>();
        let balance_ids = ids
            .iter()
            .flat_map(|id| {
                [
                    (
                        self.journal_id,
                        (*id).into(),
                        "USD".parse::<cala_ledger::Currency>().unwrap(),
                    ),
                    (
                        self.journal_id,
                        (*id).into(),
                        "BTC".parse::<cala_ledger::Currency>().unwrap(),
                    ),
                ]
            })
            .collect::<Vec<_>>();

        // Start all three queries in parallel
        let (account_sets_result, accounts_result, balances_result) = tokio::join!(
            self.cala
                .account_sets()
                .find_all::<AccountSet>(&account_set_ids),
            self.cala.accounts().find_all::<Account>(&account_ids),
            self.cala.balances().find_all(&balance_ids)
        );

        // Extract results, propagating any errors
        let account_sets = account_sets_result?;
        let accounts = accounts_result?;
        let mut balances = balances_result?;
        let mut result = HashMap::new();

        for (id, account_set) in account_sets {
            let account_id: LedgerAccountId = id.into();
            let usd_balance = balances.remove(&(
                self.journal_id,
                account_id.into(),
                "USD".parse::<cala_ledger::Currency>().unwrap(),
            ));

            let btc_balance = balances.remove(&(
                self.journal_id,
                account_id.into(),
                "BTC".parse::<cala_ledger::Currency>().unwrap(),
            ));

            let ledger_account =
                T::from(LedgerAccount::from((account_set, usd_balance, btc_balance)));
            result.insert(account_id, ledger_account);
        }

        for (id, account) in accounts {
            let account_id: LedgerAccountId = id.into();
            if result.contains_key(&account_id) {
                continue;
            }
            let usd_balance = balances.remove(&(
                self.journal_id,
                account_id.into(),
                "USD".parse::<cala_ledger::Currency>().unwrap(),
            ));

            let btc_balance = balances.remove(&(
                self.journal_id,
                account_id.into(),
                "BTC".parse::<cala_ledger::Currency>().unwrap(),
            ));

            let ledger_account = T::from(LedgerAccount::from((account, usd_balance, btc_balance)));
            result.insert(account_id, ledger_account);
        }

        Ok(result)
    }
}

impl From<(AccountSet, Option<AccountBalance>, Option<AccountBalance>)> for LedgerAccount {
    fn from(
        (account_set, usd_balance, btc_balance): (
            AccountSet,
            Option<AccountBalance>,
            Option<AccountBalance>,
        ),
    ) -> Self {
        LedgerAccount {
            id: account_set.id.into(),
            usd_balance,
            btc_balance,
        }
    }
}

impl From<(Account, Option<AccountBalance>, Option<AccountBalance>)> for LedgerAccount {
    fn from(
        (account, usd_balance, btc_balance): (
            Account,
            Option<AccountBalance>,
            Option<AccountBalance>,
        ),
    ) -> Self {
        LedgerAccount {
            id: account.id.into(),
            usd_balance,
            btc_balance,
        }
    }
}
