pub mod error;

use std::collections::HashMap;

use cala_ledger::{
    CalaLedger, Currency, JournalId,
    account::Account,
    account_set::{AccountSet, AccountSetId, AccountSetMemberId},
};

use crate::{AccountCode, LedgerAccount, LedgerAccountId, journal_error::JournalError};

use error::*;

const MAX_DEPTH_BETWEEN_LEAF_AND_COA_EDGE: usize = 2; // coa_edge -> internal_account -> leaf

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

    pub async fn ledger_account_history<T, U>(
        &self,
        ledger_account_id: LedgerAccountId,
        cursor: es_entity::PaginatedQueryArgs<U>,
    ) -> Result<es_entity::PaginatedQueryRet<T, U>, LedgerAccountLedgerError>
    where
        T: TryFrom<cala_ledger::entry::Entry, Error = JournalError>,
        U: From<cala_ledger::entry::EntriesByCreatedAtCursor> + std::fmt::Debug + Clone,
        cala_ledger::entry::EntriesByCreatedAtCursor: From<U>,
    {
        let cala_cursor_2 = es_entity::PaginatedQueryArgs {
            after: cursor
                .after
                .clone()
                .map(cala_ledger::entry::EntriesByCreatedAtCursor::from),
            first: cursor.first,
        };

        let cala_cursor = es_entity::PaginatedQueryArgs {
            after: cursor
                .after
                .map(cala_ledger::entry::EntriesByCreatedAtCursor::from),
            first: cursor.first,
        };

        let mut ret = self
            .cala
            .entries()
            .list_for_account_set_id(
                ledger_account_id.into(),
                cala_cursor,
                es_entity::ListDirection::Descending,
            )
            .await?;

        if ret.entities.is_empty() {
            ret = self
                .cala
                .entries()
                .list_for_account_id(
                    ledger_account_id.into(),
                    cala_cursor_2,
                    es_entity::ListDirection::Descending,
                )
                .await?;
        }

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

    #[allow(clippy::type_complexity)]
    pub fn find_parent_with_account_code(
        &self,
        id: AccountSetMemberId,
        current_depth: usize,
    ) -> std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<Option<(AccountSetId, AccountCode)>, LedgerAccountLedgerError>,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            if current_depth > MAX_DEPTH_BETWEEN_LEAF_AND_COA_EDGE {
                return Ok(None);
            }
            let all_parents = self
                .cala
                .account_sets()
                .find_where_member(id, Default::default())
                .await?
                .entities;

            for parent in all_parents.iter() {
                if let Some(Ok(code)) = parent
                    .values()
                    .external_id
                    .as_ref()
                    .map(|id| id.parse::<AccountCode>())
                {
                    return Ok(Some((parent.id, code)));
                }
                if let Some(res) = self
                    .find_parent_with_account_code(parent.id.into(), current_depth + 1)
                    .await?
                {
                    return Ok(Some(res));
                }
            }

            Ok(None)
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn find_leaf_children(
        &self,
        id: LedgerAccountId,
        current_depth: usize,
    ) -> std::pin::Pin<
        Box<
            dyn Future<Output = Result<Vec<LedgerAccountId>, LedgerAccountLedgerError>> + Send + '_,
        >,
    > {
        Box::pin(async move {
            if current_depth > MAX_DEPTH_BETWEEN_LEAF_AND_COA_EDGE {
                return Ok(Vec::new());
            }

            let children = self
                .cala
                .account_sets()
                .list_members_by_external_id(id.into(), Default::default())
                .await?
                .entities;

            let mut results = Vec::new();

            for child in children {
                match child.id {
                    cala_ledger::account_set::AccountSetMemberId::Account(id) => {
                        results.push(id.into());
                    }
                    cala_ledger::account_set::AccountSetMemberId::AccountSet(id) => {
                        let nested_children = self
                            .find_leaf_children(id.into(), current_depth + 1)
                            .await?;
                        results.extend(nested_children);
                    }
                }
            }

            Ok(results)
        })
    }

    pub async fn load_ledger_account_by_external_id(
        &self,
        external_id: String,
    ) -> Result<Option<LedgerAccount>, LedgerAccountLedgerError> {
        let account_set = self
            .cala
            .account_sets()
            .find_by_external_id(external_id)
            .await?;
        let balance_ids = [
            (self.journal_id, account_set.id.into(), Currency::USD),
            (self.journal_id, account_set.id.into(), Currency::BTC),
        ];
        let mut balances = self.cala.balances().find_all(&balance_ids).await?;

        let usd_balance = balances.remove(&(self.journal_id, account_set.id.into(), Currency::USD));
        let btc_balance = balances.remove(&(self.journal_id, account_set.id.into(), Currency::BTC));

        let ledger_account = LedgerAccount::from((account_set, usd_balance, btc_balance));
        Ok(Some(ledger_account))
    }

    pub async fn load_ledger_accounts(
        &self,
        ids: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, LedgerAccount>, LedgerAccountLedgerError> {
        let account_set_ids = ids.iter().map(|id| (*id).into()).collect::<Vec<_>>();
        let account_ids = ids.iter().map(|id| (*id).into()).collect::<Vec<_>>();
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
            self.cala.balances().find_all(&balance_ids)
        );

        let account_sets = account_sets_result?;
        let accounts = accounts_result?;
        let mut balances = balances_result?;
        let mut result = HashMap::new();

        for (id, account_set) in account_sets {
            let account_id: LedgerAccountId = id.into();

            let usd_balance = balances.remove(&(self.journal_id, account_id.into(), Currency::USD));
            let btc_balance = balances.remove(&(self.journal_id, account_id.into(), Currency::BTC));

            let ledger_account = LedgerAccount::from((account_set, usd_balance, btc_balance));
            result.insert(account_id, ledger_account);
        }

        for (id, account) in accounts {
            let account_id: LedgerAccountId = id.into();
            if result.contains_key(&account_id) {
                continue;
            }
            let usd_balance = balances.remove(&(self.journal_id, account_id.into(), Currency::USD));
            let btc_balance = balances.remove(&(self.journal_id, account_id.into(), Currency::BTC));

            let ledger_account = LedgerAccount::from((account, usd_balance, btc_balance));
            result.insert(account_id, ledger_account);
        }

        Ok(result)
    }
}
