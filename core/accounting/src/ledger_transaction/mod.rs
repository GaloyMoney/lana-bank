pub mod error;
mod value;

use std::collections::HashMap;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use tracing::instrument;

use crate::primitives::{CoreAccountingAction, CoreAccountingObject, LedgerTransactionId};

use error::*;
pub use value::*;

#[derive(Clone)]
pub struct LedgerTransactions<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    cala: CalaLedger,
}

impl<Perms> LedgerTransactions<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(authz: &Perms, cala: &CalaLedger) -> Self {
        Self {
            authz: authz.clone(),
            cala: cala.clone(),
        }
    }

    #[instrument(name = "accounting.ledger_transaction.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<LedgerTransactionId> + std::fmt::Debug,
    ) -> Result<Option<LedgerTransaction>, LedgerTransactionError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::ledger_transaction(id),
                CoreAccountingAction::LEDGER_TRANSACTION_READ,
            )
            .await?;

        let (transaction, entries) = tokio::join!(
            self.cala.transactions().find_by_id(id),
            self.cala.entries().list_for_transaction_id(id)
        );
        let res = match transaction {
            Ok(tx) => Some(LedgerTransaction::try_from((tx, entries?))?),
            Err(e) if e.was_not_found() => None,
            Err(e) => return Err(e.into()),
        };
        Ok(res)
    }

    pub async fn find_all<T: From<LedgerTransaction>>(
        &self,
        ids: &[LedgerTransactionId],
    ) -> Result<HashMap<LedgerTransactionId, T>, LedgerTransactionError> {
        let transactions: HashMap<_, cala_ledger::transaction::Transaction> =
            self.cala.transactions().find_all(ids).await?;
        let entries: Vec<cala_ledger::EntryId> = transactions
            .values()
            .flat_map(|tx| tx.values().entry_ids.iter().copied())
            .collect();

        let mut all_entries: HashMap<_, cala_ledger::entry::Entry> =
            self.cala.entries().find_all(&entries).await?;

        let mut res = HashMap::new();

        for (tx_id, tx) in transactions {
            let tx_entries: Vec<_> = tx
                .values()
                .entry_ids
                .iter()
                .filter_map(|entry_id| all_entries.remove(entry_id))
                .collect();

            let mut sorted_entries = tx_entries;
            sorted_entries.sort_by(|a, b| {
                let a_sequence = a.values().sequence;
                let b_sequence = b.values().sequence;
                a_sequence.cmp(&b_sequence)
            });

            match LedgerTransaction::try_from((tx, sorted_entries)) {
                Ok(ledger_tx) => {
                    res.insert(tx_id, T::from(ledger_tx));
                }
                Err(e) => return Err(e),
            }
        }

        Ok(res)
    }
}
