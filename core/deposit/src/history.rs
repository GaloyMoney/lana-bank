use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::primitives::{CalaEntryId, CalaTransactionId as CalaTxId};

pub enum DepositAccountHistoryEntry {
    Deposit(DepositEntry),
    Withdrawal(WithdrawalEntry),
    CancelledWithdrawal(WithdrawalEntry),
    Disbursal(DisbursalEntry),
    Payment(PaymentEntry),
    Unknown(UnknownEntry),
    Ignored,
}
pub struct DepositEntry {
    pub tx_id: CalaTxId,
    pub entry_id: CalaEntryId,
    pub recorded_at: DateTime<Utc>,
}

pub struct WithdrawalEntry {
    pub tx_id: CalaTxId,
    pub entry_id: CalaEntryId,
    pub recorded_at: DateTime<Utc>,
}

pub struct DisbursalEntry {
    pub tx_id: CalaTxId,
    pub entry_id: CalaEntryId,
    pub recorded_at: DateTime<Utc>,
}

pub struct PaymentEntry {
    pub tx_id: CalaTxId,
    pub entry_id: CalaEntryId,
    pub recorded_at: DateTime<Utc>,
}

pub struct UnknownEntry {
    pub tx_id: CalaTxId,
    pub entry_id: CalaEntryId,
    pub recorded_at: DateTime<Utc>,
}

const RECORD_DEPOSIT: &str = "RECORD_DEPOSIT_CR";
const INITIATE_WITHDRAW: &str = "INITIATE_WITHDRAW_SETTLED_DR";
const CANCEL_WITHDRAW: &str = "CANCEL_WITHDRAW_SETTLED_CR";
const CONFIRM_DISBURSAL: &str = "CONFIRM_DISBURSAL_SETTLED_CR";
const RECORD_PAYMENT_ALLOCATION: &str = "RECORD_PAYMENT_ALLOCATION_DR";

const IGNORE_INITIATE_WITHDRAW_PENDING: &str = "INITIATE_WITHDRAW_PENDING_CR";
const IGNORE_CONFIRM_WITHDRAWAL_PENDING: &str = "CONFIRM_WITHDRAW_PENDING_DR";
const IGNORE_CANCEL_WITHDRAW_PENDING: &str = "CANCEL_WITHDRAW_PENDING_DR";

impl From<cala_ledger::entry::Entry> for DepositAccountHistoryEntry {
    fn from(entry: cala_ledger::entry::Entry) -> Self {
        match entry.values().entry_type.as_str() {
            RECORD_DEPOSIT => DepositAccountHistoryEntry::Deposit(DepositEntry {
                tx_id: entry.values().transaction_id,
                entry_id: entry.id,
                recorded_at: entry.created_at(),
            }),
            INITIATE_WITHDRAW => DepositAccountHistoryEntry::Withdrawal(WithdrawalEntry {
                tx_id: entry.values().transaction_id,
                entry_id: entry.id,
                recorded_at: entry.created_at(),
            }),
            CANCEL_WITHDRAW => DepositAccountHistoryEntry::CancelledWithdrawal(WithdrawalEntry {
                tx_id: entry.values().transaction_id,
                entry_id: entry.id,
                recorded_at: entry.created_at(),
            }),
            CONFIRM_DISBURSAL => DepositAccountHistoryEntry::Disbursal(DisbursalEntry {
                tx_id: entry.values().transaction_id,
                entry_id: entry.id,
                recorded_at: entry.created_at(),
            }),
            RECORD_PAYMENT_ALLOCATION => DepositAccountHistoryEntry::Payment(PaymentEntry {
                tx_id: entry.values().transaction_id,
                entry_id: entry.id,
                recorded_at: entry.created_at(),
            }),

            IGNORE_CONFIRM_WITHDRAWAL_PENDING => DepositAccountHistoryEntry::Ignored,
            IGNORE_INITIATE_WITHDRAW_PENDING => DepositAccountHistoryEntry::Ignored,
            IGNORE_CANCEL_WITHDRAW_PENDING => DepositAccountHistoryEntry::Ignored,

            _ => DepositAccountHistoryEntry::Unknown(UnknownEntry {
                tx_id: entry.values().transaction_id,
                entry_id: entry.id,
                recorded_at: entry.created_at(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAccountHistoryCursor {
    pub entry_id: CalaEntryId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&DepositAccountHistoryEntry> for DepositAccountHistoryCursor {
    fn from(entry: &DepositAccountHistoryEntry) -> Self {
        match entry {
            DepositAccountHistoryEntry::Deposit(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.recorded_at,
            },
            DepositAccountHistoryEntry::Withdrawal(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.recorded_at,
            },
            DepositAccountHistoryEntry::CancelledWithdrawal(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.recorded_at,
            },
            DepositAccountHistoryEntry::Disbursal(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.recorded_at,
            },
            DepositAccountHistoryEntry::Payment(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.recorded_at,
            },
            DepositAccountHistoryEntry::Unknown(entry) => Self {
                entry_id: entry.entry_id,
                created_at: entry.recorded_at,
            },
            DepositAccountHistoryEntry::Ignored => {
                unreachable!("Ignored entries should not be cursorized")
            }
        }
    }
}

impl From<cala_ledger::entry::EntriesByCreatedAtCursor> for DepositAccountHistoryCursor {
    fn from(cursor: cala_ledger::entry::EntriesByCreatedAtCursor) -> Self {
        Self {
            entry_id: cursor.id,
            created_at: cursor.created_at,
        }
    }
}

impl From<DepositAccountHistoryCursor> for cala_ledger::entry::EntriesByCreatedAtCursor {
    fn from(cursor: DepositAccountHistoryCursor) -> Self {
        Self {
            id: cursor.entry_id,
            created_at: cursor.created_at,
        }
    }
}

#[cfg(feature = "graphql")]
mod graphql {
    use async_graphql::{connection::CursorType, *};

    use super::*;

    impl CursorType for DepositAccountHistoryCursor {
        type Error = String;

        fn encode_cursor(&self) -> String {
            use base64::{engine::general_purpose, Engine as _};
            let json = serde_json::to_string(&self).expect("could not serialize cursor");
            general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
        }

        fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
            use base64::{engine::general_purpose, Engine as _};
            let bytes = general_purpose::STANDARD_NO_PAD
                .decode(s.as_bytes())
                .map_err(|e| e.to_string())?;
            let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
            serde_json::from_str(&json).map_err(|e| e.to_string())
        }
    }
}
