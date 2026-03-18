use serde::{Deserialize, Serialize};

use money::{CurrencyCode, Satoshis, UsdCents};

use crate::primitives::RestrictedCurrencyMap;

/// Balance for a single currency within a deposit account.
///
/// Each variant preserves the type safety of its underlying minor-units type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "currency")]
pub enum DepositAccountBalance {
    #[serde(rename = "USD")]
    Usd {
        settled: UsdCents,
        pending: UsdCents,
    },
    #[serde(rename = "BTC")]
    Btc {
        settled: Satoshis,
        pending: Satoshis,
    },
}

impl DepositAccountBalance {
    pub fn currency(&self) -> CurrencyCode {
        match self {
            Self::Usd { .. } => CurrencyCode::USD,
            Self::Btc { .. } => CurrencyCode::BTC,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Usd { settled, pending } => settled.is_zero() && pending.is_zero(),
            Self::Btc { settled, pending } => settled.is_zero() && pending.is_zero(),
        }
    }

    pub fn zero_usd() -> Self {
        Self::Usd {
            settled: UsdCents::ZERO,
            pending: UsdCents::ZERO,
        }
    }

    pub fn zero_btc() -> Self {
        Self::Btc {
            settled: Satoshis::ZERO,
            pending: Satoshis::ZERO,
        }
    }
}

/// Per-currency balances for a deposit account, restricted to the account's currency scope.
pub type DepositAccountBalances = RestrictedCurrencyMap<DepositAccountBalance>;
