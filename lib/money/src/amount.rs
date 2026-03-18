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

/// A type-safe balance for a single currency, with settled and pending layers.
///
/// Like `CurrencyAmount`, each variant preserves the precision of its underlying type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "currency")]
pub enum CurrencyBalance {
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

impl CurrencyBalance {
    pub fn currency(&self) -> CurrencyCode {
        match self {
            Self::Usd { .. } => <Usd as crate::Currency>::CODE,
            Self::Btc { .. } => <Btc as crate::Currency>::CODE,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Usd { settled, pending } => settled.is_zero() && pending.is_zero(),
            Self::Btc { settled, pending } => settled.is_zero() && pending.is_zero(),
        }
    }

    pub fn settled_major(&self) -> Decimal {
        match self {
            Self::Usd { settled, .. } => settled.to_major(),
            Self::Btc { settled, .. } => settled.to_major(),
        }
    }

    pub fn pending_major(&self) -> Decimal {
        match self {
            Self::Usd { pending, .. } => pending.to_major(),
            Self::Btc { pending, .. } => pending.to_major(),
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

#[cfg(feature = "sqlx")]
mod currency_amount_sqlx {
    use sqlx::{Type, postgres::*};

    use super::CurrencyAmount;

    impl Type<Postgres> for CurrencyAmount {
        fn type_info() -> PgTypeInfo {
            <sqlx::types::Json<CurrencyAmount> as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <sqlx::types::Json<CurrencyAmount> as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for CurrencyAmount {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <sqlx::types::Json<&CurrencyAmount> as sqlx::Encode<'_, Postgres>>::encode(
                sqlx::types::Json(self),
                buf,
            )
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for CurrencyAmount {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let json =
                <sqlx::types::Json<CurrencyAmount> as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(json.0)
        }
    }

    impl PgHasArrayType for CurrencyAmount {
        fn array_type_info() -> PgTypeInfo {
            <sqlx::types::Json<CurrencyAmount> as PgHasArrayType>::array_type_info()
        }
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
    fn currency_balance_zero_check() {
        let zero = CurrencyBalance::zero_usd();
        assert!(zero.is_zero());

        let non_zero = CurrencyBalance::Usd {
            settled: UsdCents::from(100u64),
            pending: UsdCents::ZERO,
        };
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn serde_roundtrip() {
        let amt = CurrencyAmount::usd(UsdCents::from(42u64));
        let json = serde_json::to_string(&amt).unwrap();
        let back: CurrencyAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(amt, back);
    }
}
