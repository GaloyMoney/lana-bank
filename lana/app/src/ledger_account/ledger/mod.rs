pub mod error;

use cala_ledger::{AccountSetId, CalaLedger, JournalId};

use error::*;

#[derive(Clone)]
pub struct LedgerAccountLedger {
    cala: CalaLedger,
    _journal_id: JournalId,
}

impl LedgerAccountLedger {
    pub fn init(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            _journal_id: journal_id, // TODO: filter entries by journal_id
        }
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
