use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{Btc, CurrencyCode, Satoshis, Usd, UsdCents};

/// A type-safe amount tagged with its currency.
///
/// Bridges runtime currency dispatch (`CurrencyCode`) with compile-time type
/// safety (`UsdCents`, `Satoshis`). Each variant preserves the precision and
/// semantics of its underlying `MinorUnits<C>` type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "currency", content = "minor_units")]
pub enum CurrencyAmount {
    #[serde(rename = "USD")]
    Usd(UsdCents),
    #[serde(rename = "BTC")]
    Btc(Satoshis),
}

impl CurrencyAmount {
    pub fn currency(&self) -> CurrencyCode {
        match self {
            Self::Usd(_) => <Usd as crate::Currency>::CODE,
            Self::Btc(_) => <Btc as crate::Currency>::CODE,
        }
    }

    pub fn to_major(self) -> Decimal {
        match self {
            Self::Usd(v) => v.to_major(),
            Self::Btc(v) => v.to_major(),
        }
    }

    pub fn is_zero(self) -> bool {
        match self {
            Self::Usd(v) => v.is_zero(),
            Self::Btc(v) => v.is_zero(),
        }
    }

    pub fn usd(cents: UsdCents) -> Self {
        Self::Usd(cents)
    }

    pub fn btc(sats: Satoshis) -> Self {
        Self::Btc(sats)
    }

    pub fn as_usd(self) -> Option<UsdCents> {
        match self {
            Self::Usd(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_btc(self) -> Option<Satoshis> {
        match self {
            Self::Btc(v) => Some(v),
            _ => None,
        }
    }
}

impl From<UsdCents> for CurrencyAmount {
    fn from(v: UsdCents) -> Self {
        Self::Usd(v)
    }
}

impl From<Satoshis> for CurrencyAmount {
    fn from(v: Satoshis) -> Self {
        Self::Btc(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_amount_preserves_type() {
        let usd = CurrencyAmount::usd(UsdCents::from(1000u64));
        assert_eq!(usd.currency(), CurrencyCode::USD);
        assert_eq!(usd.as_usd(), Some(UsdCents::from(1000u64)));
        assert_eq!(usd.as_btc(), None);

        let btc = CurrencyAmount::btc(Satoshis::from(50_000_000u64));
        assert_eq!(btc.currency(), CurrencyCode::BTC);
        assert_eq!(btc.as_btc(), Some(Satoshis::from(50_000_000u64)));
    }

    #[test]
    fn serde_roundtrip() {
        let amt = CurrencyAmount::usd(UsdCents::from(42u64));
        let json = serde_json::to_string(&amt).unwrap();
        let back: CurrencyAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(amt, back);
    }
}
