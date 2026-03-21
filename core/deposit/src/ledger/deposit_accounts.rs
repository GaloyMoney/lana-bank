#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

use crate::primitives::RestrictedCurrencyMap;

/// A pair of CALA ledger accounts for a single currency: one active, one frozen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct LedgerAccountPair {
    pub active: CalaAccountId,
    pub frozen: CalaAccountId,
}

impl LedgerAccountPair {
    pub fn new(active: CalaAccountId, frozen: CalaAccountId) -> Self {
        Self { active, frozen }
    }
}

/// Per-account ledger IDs: one `LedgerAccountPair` per currency in the account's scope.
///
/// The `RestrictedCurrencyMap` enforces that only currencies in the account's
/// allowed set can be accessed. The allowed set is the account's "currency scope".
pub type DepositAccountLedgerAccountIds = RestrictedCurrencyMap<LedgerAccountPair>;
