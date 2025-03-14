use cala_ledger::{DebitOrCredit, Layer};
use serde::{Deserialize, Serialize};

use crate::primitives::{LedgerAccountId, LedgerEntryId, LedgerTxId, Satoshis, UsdCents};

use super::ledger::error::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerAccountHistoryCursor {
    pub entry_id: LedgerEntryId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LedgerAccountHistoryCursor> for cala_ledger::entry::EntriesByCreatedAtCursor {
    fn from(cursor: LedgerAccountHistoryCursor) -> Self {
        Self {
            id: cursor.entry_id,
            created_at: cursor.created_at,
        }
    }
}

impl From<LedgerAccountEntry> for LedgerAccountHistoryCursor {
    fn from(cursor: LedgerAccountEntry) -> Self {
        Self {
            entry_id: cursor.entry_id,
            created_at: cursor.recorded_at,
        }
    }
}

impl From<&LedgerAccountEntry> for LedgerAccountHistoryCursor {
    fn from(cursor: &LedgerAccountEntry) -> Self {
        Self {
            entry_id: cursor.entry_id,
            created_at: cursor.recorded_at,
        }
    }
}

impl From<cala_ledger::entry::EntriesByCreatedAtCursor> for LedgerAccountHistoryCursor {
    fn from(cursor: cala_ledger::entry::EntriesByCreatedAtCursor) -> Self {
        Self {
            entry_id: cursor.id,
            created_at: cursor.created_at,
        }
    }
}

impl es_entity::graphql::async_graphql::connection::CursorType for LedgerAccountHistoryCursor {
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

pub struct LedgerAccountEntry {
    pub tx_id: LedgerTxId,
    pub entry_id: LedgerEntryId,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
    pub account_id: LedgerAccountId,
    pub entry_type: String,
    pub amount: LayeredLedgerAccountAmount,
}

impl TryFrom<cala_ledger::entry::Entry> for LedgerAccountEntry {
    type Error = LedgerAccountLedgerError;

    fn try_from(cala_entry: cala_ledger::entry::Entry) -> Result<Self, Self::Error> {
        let currency = cala_entry.values().currency;

        let mut layered_amount = None;

        let mut layered_usd_amount = LayeredUsdLedgerAccountAmount::ZERO;
        if currency == "USD".parse()? {
            let mut usd_amount = UsdLedgerAccountAmount::ZERO;
            match cala_entry.values().direction {
                DebitOrCredit::Debit => {
                    usd_amount.dr_amount = UsdCents::try_from_usd(cala_entry.values().units)?
                }
                DebitOrCredit::Credit => {
                    usd_amount.cr_amount = UsdCents::try_from_usd(cala_entry.values().units)?
                }
            }
            match cala_entry.values().layer {
                Layer::Settled => layered_usd_amount.settled = usd_amount,
                Layer::Pending => layered_usd_amount.pending = usd_amount,
                Layer::Encumbrance => layered_usd_amount.encumbrance = usd_amount,
            }

            layered_amount = Some(LayeredLedgerAccountAmount::Usd(layered_usd_amount))
        }

        let mut layered_btc_amount = LayeredBtcLedgerAccountAmount::ZERO;
        if currency == "BTC".parse()? {
            let mut btc_amount = BtcLedgerAccountAmount::ZERO;
            match cala_entry.values().direction {
                DebitOrCredit::Debit => {
                    btc_amount.dr_amount = Satoshis::try_from_btc(cala_entry.values().units)?
                }
                DebitOrCredit::Credit => {
                    btc_amount.cr_amount = Satoshis::try_from_btc(cala_entry.values().units)?
                }
            }
            match cala_entry.values().layer {
                Layer::Settled => layered_btc_amount.settled = btc_amount,
                Layer::Pending => layered_btc_amount.pending = btc_amount,
                Layer::Encumbrance => layered_btc_amount.encumbrance = btc_amount,
            }

            layered_amount = Some(LayeredLedgerAccountAmount::Btc(layered_btc_amount))
        }

        Ok(Self {
            tx_id: cala_entry.values().transaction_id,
            entry_id: cala_entry.id,
            recorded_at: cala_entry.created_at(),
            account_id: cala_entry.values().account_id,
            entry_type: cala_entry.values().entry_type.to_string(),
            amount: layered_amount.expect("Currency is not 'USD' or 'BTC'"),
        })
    }
}

pub enum LayeredLedgerAccountAmount {
    Usd(LayeredUsdLedgerAccountAmount),
    Btc(LayeredBtcLedgerAccountAmount),
}

pub struct LayeredUsdLedgerAccountAmount {
    pub settled: UsdLedgerAccountAmount,
    pub pending: UsdLedgerAccountAmount,
    pub encumbrance: UsdLedgerAccountAmount,
}

impl LayeredUsdLedgerAccountAmount {
    pub const ZERO: Self = Self {
        settled: UsdLedgerAccountAmount::ZERO,
        pending: UsdLedgerAccountAmount::ZERO,
        encumbrance: UsdLedgerAccountAmount::ZERO,
    };
}

pub struct UsdLedgerAccountAmount {
    pub dr_amount: UsdCents,
    pub cr_amount: UsdCents,
}

impl UsdLedgerAccountAmount {
    pub const ZERO: Self = Self {
        dr_amount: UsdCents::ZERO,
        cr_amount: UsdCents::ZERO,
    };
}

pub struct LayeredBtcLedgerAccountAmount {
    pub settled: BtcLedgerAccountAmount,
    pub pending: BtcLedgerAccountAmount,
    pub encumbrance: BtcLedgerAccountAmount,
}

impl LayeredBtcLedgerAccountAmount {
    pub const ZERO: Self = Self {
        settled: BtcLedgerAccountAmount::ZERO,
        pending: BtcLedgerAccountAmount::ZERO,
        encumbrance: BtcLedgerAccountAmount::ZERO,
    };
}

pub struct BtcLedgerAccountAmount {
    pub dr_amount: Satoshis,
    pub cr_amount: Satoshis,
}

impl BtcLedgerAccountAmount {
    pub const ZERO: Self = Self {
        dr_amount: Satoshis::ZERO,
        cr_amount: Satoshis::ZERO,
    };
}
