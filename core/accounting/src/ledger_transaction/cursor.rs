use serde::{Deserialize, Serialize};

use cala_ledger::transaction::TransactionsByCreatedAtCursor;

use super::LedgerTransaction;
use crate::primitives::LedgerTransactionId;

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

impl<S> From<&LedgerTransaction<S>> for LedgerTransactionCursor {
    fn from(transaction: &LedgerTransaction<S>) -> Self {
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
