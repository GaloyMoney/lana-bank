pub mod error;

use std::collections::HashMap;

use cala_ledger::{
    CalaLedger, Currency, JournalId,
    account::Account,
    account_set::{AccountSet, AccountSetId, AccountSetMemberId},
};
use chrono::NaiveDate;
use tracing::instrument;

use crate::{AccountCode, LedgerAccount, LedgerAccountId, journal_error::JournalError};

use super::{AccountBalances, BalanceRanges};

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
                match (child.external_id, child.id) {
                    (
                        Some(external_id),
                        cala_ledger::account_set::AccountSetMemberId::AccountSet(id),
                    ) if external_id.parse::<AccountCode>().is_ok() => {
                        results.push(id.into());
                    }
                    (_, cala_ledger::account_set::AccountSetMemberId::Account(id)) => {
                        results.push(id.into());
                    }
                    (_, cala_ledger::account_set::AccountSetMemberId::AccountSet(id)) => {
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

    #[instrument(name = "ledger_account.load_by_external_id", skip(self), fields(external_id = %external_id))]
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

        let account_balances =
            AccountBalances::extract_from_balances(&mut balances, self.journal_id, account_set.id);

        let ledger_account = LedgerAccount::from((account_set, account_balances));
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

            let account_balances =
                AccountBalances::extract_from_balances(&mut balances, self.journal_id, account_id);

            let ledger_account = LedgerAccount::from((account_set, account_balances));
            result.insert(account_id, ledger_account);
        }

        for (id, account) in accounts {
            let account_id: LedgerAccountId = id.into();
            if result.contains_key(&account_id) {
                continue;
            }
            let account_balances =
                AccountBalances::extract_from_balances(&mut balances, self.journal_id, account_id);

            let ledger_account = LedgerAccount::from((account, account_balances));
            result.insert(account_id, ledger_account);
        }

        Ok(result)
    }

    pub async fn load_account_sets_in_range(
        &self,
        ids: &[LedgerAccountId],
        from: NaiveDate,
        until: Option<NaiveDate>,
        filter_non_zero: bool,
    ) -> Result<Vec<LedgerAccount>, LedgerAccountLedgerError> {
        let account_set_ids: Vec<AccountSetId> = ids.iter().map(|id| (*id).into()).collect();
        let balance_ids = ids
            .iter()
            .flat_map(|id| {
                [
                    (self.journal_id, (*id).into(), Currency::USD),
                    (self.journal_id, (*id).into(), Currency::BTC),
                ]
            })
            .collect::<Vec<_>>();

        let (account_sets_result, balances_result) = tokio::join!(
            self.cala
                .account_sets()
                .find_all::<AccountSet>(&account_set_ids),
            self.cala
                .balances()
                .effective()
                .find_all_in_range(&balance_ids, from, until)
        );

        let mut account_sets = account_sets_result?;
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
                let balance_ranges = BalanceRanges {
                    btc: btc_balance,
                    usd: usd_balance,
                };
                let row = LedgerAccount::from((account_set, balance_ranges));
                if filter_non_zero && !row.has_non_zero_activity() {
                    continue;
                }
                rows.push(row);
            }
        }

        Ok(rows)
    }

    /// Batch load parent account information for multiple account set members.
    /// Returns a map from member UUID to (AccountSetId, AccountCode) for parents with codes.
    /// The key is the UUID representation of the member (either account_id or account_set_id).
    #[instrument(name = "ledger_account.batch_find_parents", skip(self))]
    pub async fn batch_find_parents_with_account_code(
        &self,
        member_ids: &[AccountSetMemberId],
    ) -> Result<HashMap<uuid::Uuid, (AccountSetId, AccountCode)>, LedgerAccountLedgerError> {
        let mut result = HashMap::new();
        let mut to_process: Vec<(AccountSetMemberId, uuid::Uuid, usize)> = member_ids
            .iter()
            .map(|id| {
                let uuid = match id {
                    AccountSetMemberId::Account(aid) => uuid::Uuid::from(aid),
                    AccountSetMemberId::AccountSet(asid) => uuid::Uuid::from(asid),
                };
                (*id, uuid, 0)
            })
            .collect();
        let mut processed_uuids: Vec<uuid::Uuid> = Vec::new();

        while !to_process.is_empty() {
            let current_batch: Vec<_> = to_process.drain(..).collect();
            let mut next_batch = Vec::new();

            let futures: Vec<_> = current_batch
                .iter()
                .filter(|(_, uuid, _)| !processed_uuids.contains(uuid))
                .map(|(id, uuid, depth)| async move {
                    if *depth > MAX_DEPTH_BETWEEN_LEAF_AND_COA_EDGE {
                        return Ok((*id, *uuid, Vec::new()));
                    }
                    let parents = self
                        .cala
                        .account_sets()
                        .find_where_member(*id, Default::default())
                        .await?
                        .entities;
                    Ok::<_, LedgerAccountLedgerError>((*id, *uuid, parents))
                })
                .collect();

            let results = futures::future::join_all(futures).await;

            for res in results {
                let (_member_id, member_uuid, parents) = res?;
                processed_uuids.push(member_uuid);

                for parent in parents {
                    if let Some(Ok(code)) = parent
                        .values()
                        .external_id
                        .as_ref()
                        .map(|id| id.parse::<AccountCode>())
                    {
                        result.insert(member_uuid, (parent.id, code));
                        break;
                    } else {
                        let parent_member_id = parent.id.into();
                        let parent_uuid = uuid::Uuid::from(&parent.id);
                        if !processed_uuids.contains(&parent_uuid)
                            && !result.contains_key(&member_uuid)
                        {
                            let depth = current_batch
                                .iter()
                                .find(|(_, u, _)| *u == member_uuid)
                                .map(|(_, _, d)| *d)
                                .unwrap_or(0);
                            next_batch.push((parent_member_id, parent_uuid, depth + 1));
                        }
                    }
                }
            }

            to_process = next_batch;
        }

        Ok(result)
    }

    /// Batch load child account information for multiple account sets.
    /// Returns a map from LedgerAccountId to Vec<LedgerAccountId> of leaf children.
    #[instrument(name = "ledger_account.batch_find_children", skip(self))]
    pub async fn batch_find_leaf_children(
        &self,
        account_ids: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, Vec<LedgerAccountId>>, LedgerAccountLedgerError> {
        let mut result: HashMap<LedgerAccountId, Vec<LedgerAccountId>> = HashMap::new();
        let mut to_process: Vec<(LedgerAccountId, usize)> =
            account_ids.iter().map(|id| (*id, 0)).collect();

        while !to_process.is_empty() {
            let current_batch: Vec<_> = to_process.drain(..).collect();
            let mut next_batch = Vec::new();

            let futures: Vec<_> = current_batch
                .iter()
                .map(|(id, depth)| async move {
                    if *depth > MAX_DEPTH_BETWEEN_LEAF_AND_COA_EDGE {
                        return Ok((*id, Vec::new()));
                    }
                    let children = self
                        .cala
                        .account_sets()
                        .list_members_by_external_id((*id).into(), Default::default())
                        .await?
                        .entities;
                    Ok::<_, LedgerAccountLedgerError>((*id, children))
                })
                .collect();

            let results = futures::future::join_all(futures).await;

            for res in results {
                let (parent_id, children) = res?;

                // Collect leaf children and nested account sets separately to avoid borrow conflicts
                let mut leaf_children: Vec<LedgerAccountId> = Vec::new();
                let mut nested_sets: Vec<LedgerAccountId> = Vec::new();

                for child in children {
                    match (child.external_id, child.id) {
                        (
                            Some(external_id),
                            cala_ledger::account_set::AccountSetMemberId::AccountSet(id),
                        ) if external_id.parse::<AccountCode>().is_ok() => {
                            leaf_children.push(id.into());
                        }
                        (_, cala_ledger::account_set::AccountSetMemberId::Account(id)) => {
                            leaf_children.push(id.into());
                        }
                        (_, cala_ledger::account_set::AccountSetMemberId::AccountSet(id)) => {
                            nested_sets.push(id.into());
                        }
                    }
                }

                // Now update the result map
                let entry = result.entry(parent_id).or_default();
                entry.extend(leaf_children);

                // Process nested sets for next iteration
                let depth = current_batch
                    .iter()
                    .find(|(pid, _)| *pid == parent_id)
                    .map(|(_, d)| *d)
                    .unwrap_or(0);

                for child_id in nested_sets {
                    if !result.contains_key(&child_id) {
                        next_batch.push((child_id, depth + 1));
                    }
                }
            }

            to_process = next_batch;
        }

        Ok(result)
    }
}
