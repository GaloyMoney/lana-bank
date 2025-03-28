pub mod error;

use cala_ledger::{AccountSetId, CalaLedger, JournalId};

use error::*;

#[derive(Clone)]
pub struct LedgerAccountLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl LedgerAccountLedger {
    pub fn init(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id, // TODO: filter entries by journal_id
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
}
