mod entry;
pub mod error;

pub use entry::*;
use error::*;

use audit::AuditSvc;
use authz::PermissionCheck;

use cala_ledger::CalaLedger;

use crate::primitives::{CalaJournalId, CoreAccountingAction, CoreAccountingObject};

#[derive(Clone)]
pub struct Journal<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    cala: CalaLedger,
    journal_id: CalaJournalId,
}

impl<Perms> Journal<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(authz: &Perms, cala: &CalaLedger, journal_id: CalaJournalId) -> Self {
        Self {
            authz: authz.clone(),
            cala: cala.clone(),
            journal_id,
        }
    }

    pub async fn entries(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        args: es_entity::PaginatedQueryArgs<JournalEntryCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<JournalEntry, JournalEntryCursor>, JournalError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::journal(self.journal_id),
                CoreAccountingAction::JOURNAL_READ_ENTRIES,
            )
            .await?;

        let cala_cursor = es_entity::PaginatedQueryArgs {
            after: args
                .after
                .map(cala_ledger::entry::EntriesByCreatedAtCursor::from),
            first: args.first,
        };

        let ret = self
            .cala
            .entries()
            .list_for_journal_id(
                self.journal_id,
                cala_cursor,
                es_entity::ListDirection::Descending,
            )
            .await?;

        let entities = ret
            .entities
            .into_iter()
            .map(JournalEntry::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(es_entity::PaginatedQueryRet {
            entities,
            has_next_page: ret.has_next_page,
            end_cursor: ret.end_cursor.map(JournalEntryCursor::from),
        })
    }
}
