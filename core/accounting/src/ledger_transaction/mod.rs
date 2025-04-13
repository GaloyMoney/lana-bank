pub mod error;
mod value;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{CalaLedger, transaction::TransactionsByCreatedAtCursor};
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
        self.transactions_into_ledger_transactions(self.cala.transactions().find_all(ids).await?)
            .await
    }

    #[instrument(
        name = "accounting.ledger_transaction.find_by_template_code",
        skip(self),
        err
    )]
    pub async fn find_by_template_code(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        template_code: String,
        args: es_entity::PaginatedQueryArgs<LedgerTransactionCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<LedgerTransaction, LedgerTransactionCursor>,
        LedgerTransactionError,
    > {
        let template = self
            .cala
            .tx_templates()
            .find_by_code(&template_code)
            .await?;

        let cala_cursor = es_entity::PaginatedQueryArgs {
            after: args.after.map(TransactionsByCreatedAtCursor::from),
            first: args.first,
        };

        let transactions = self
            .cala
            .transactions()
            .list_for_template_id(template.id, cala_cursor, Default::default())
            .await?;

        let entities = self
            .transactions_into_ledger_transactions(
                transactions
                    .entities
                    .into_iter()
                    .map(|tx| (tx.id, tx))
                    .collect(),
            )
            .await?
            .into_values()
            .collect();

        Ok(es_entity::PaginatedQueryRet {
            entities,
            has_next_page: transactions.has_next_page,
            end_cursor: transactions.end_cursor.map(LedgerTransactionCursor::from),
        })
    }

    async fn transactions_into_ledger_transactions<T: From<LedgerTransaction>>(
        &self,
        transactions: HashMap<cala_ledger::TransactionId, cala_ledger::transaction::Transaction>,
    ) -> Result<HashMap<LedgerTransactionId, T>, LedgerTransactionError> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerTransactionCursor {
    pub ledger_transaction_id: LedgerTransactionId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<TransactionsByCreatedAtCursor> for LedgerTransactionCursor {
    fn from(cursor: TransactionsByCreatedAtCursor) -> Self {
        Self {
            ledger_transaction_id: cursor.id,
            created_at: cursor.created_at,
        }
    }
}

impl From<LedgerTransactionCursor> for TransactionsByCreatedAtCursor {
    fn from(cursor: LedgerTransactionCursor) -> Self {
        Self {
            id: cursor.ledger_transaction_id,
            created_at: cursor.created_at,
        }
    }
}

impl From<&LedgerTransaction> for LedgerTransactionCursor {
    fn from(transaction: &LedgerTransaction) -> Self {
        Self {
            ledger_transaction_id: transaction.id,
            created_at: transaction.created_at,
        }
    }
}

#[cfg(feature = "graphql")]
impl async_graphql::connection::CursorType for LedgerTransactionCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{Engine as _, engine::general_purpose};
        let json = serde_json::to_string(&self).expect("could not serialize cursor");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}
