use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    Currency, CurrencyCode, CurrencyMapError, CurrencySet, MinorUnits, RestrictedCurrencyMap,
};

/// A type-erased currency amount with compile-time safe construction.
///
/// All fields are private — the only way to create an `Amount` is from a typed
/// `MinorUnits<C>`, which captures `CurrencyCode` and `MINOR_UNITS_PER_MAJOR`
/// from the `Currency` trait at compile time. This guarantees the currency
/// metadata is always consistent with the stored value.
///
/// To recover the typed `MinorUnits<C>`, use [`to_minor_units`](Self::to_minor_units).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    currency: CurrencyCode,
    minor_units: u64,
    minor_units_per_major: u64,
}

impl Amount {
    /// Construct from a typed `MinorUnits<C>`.
    ///
    /// Captures `C::CODE` and `C::MINOR_UNITS_PER_MAJOR` at compile time,
    /// guaranteeing consistency.
    pub fn from_minor_units<C: Currency>(value: MinorUnits<C>) -> Self {
        Self {
            currency: C::CODE,
            minor_units: value.into_inner(),
            minor_units_per_major: C::MINOR_UNITS_PER_MAJOR,
        }
    }

    /// Downcast back to typed `MinorUnits<C>`.
    ///
    /// Returns `Some` only if `C::CODE` matches the stored currency.
    pub fn to_minor_units<C: Currency>(&self) -> Option<MinorUnits<C>> {
        (self.currency == C::CODE).then(|| MinorUnits::from(self.minor_units))
    }

    pub fn currency(&self) -> CurrencyCode {
        self.currency
    }

    pub fn is_zero(self) -> bool {
        self.minor_units == 0
    }

    /// Convert to major units using the captured scale factor.
    pub fn to_major(self) -> Decimal {
        Decimal::from(self.minor_units) / Decimal::from(self.minor_units_per_major)
    }
}

/// Ergonomic construction: `Amount::from(UsdCents::from(100u64))`
impl<C: Currency> From<MinorUnits<C>> for Amount {
    fn from(value: MinorUnits<C>) -> Self {
        Self::from_minor_units(value)
    }
}

// ---------------------------------------------------------------------------
// Amounts — typed collection of per-currency amounts
// ---------------------------------------------------------------------------

/// A restricted collection of `Amount` values keyed by currency.
///
/// Wraps `RestrictedCurrencyMap<Amount>` with typed accessors that use
/// `C: Currency` to ensure compile-time safety on insert and retrieval.
///
/// Used for deposit amounts, balances, or any per-currency value storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amounts(RestrictedCurrencyMap<Amount>);

impl Amounts {
    pub fn new(allowed: impl IntoIterator<Item = CurrencyCode>) -> Self {
        Self(RestrictedCurrencyMap::new(allowed))
    }

    pub fn allowed_currencies(&self) -> &CurrencySet {
        self.0.allowed_currencies()
    }

    /// Type-safe insert: captures currency from `C` at compile time.
    pub fn insert<C: Currency>(
        &mut self,
        value: MinorUnits<C>,
    ) -> Result<Option<Amount>, CurrencyMapError> {
        self.0.insert(C::CODE, Amount::from(value))
    }

    /// Type-safe get: returns typed `MinorUnits<C>`.
    pub fn get<C: Currency>(&self) -> Result<Option<MinorUnits<C>>, CurrencyMapError> {
        match self.0.get(&C::CODE)? {
            Some(amt) => Ok(amt.to_minor_units::<C>()),
            None => Ok(None),
        }
    }

    /// Get the raw `Amount` by currency code (for runtime dispatch).
    pub fn get_by_code(
        &self,
        currency: &CurrencyCode,
    ) -> Result<Option<&Amount>, CurrencyMapError> {
        self.0.get(currency)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CurrencyCode, &Amount)> {
        self.0.iter()
    }

    pub fn is_zero(&self) -> bool {
        self.0.values().all(|amt| amt.is_zero())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(feature = "sqlx")]
mod amount_sqlx {
    use sqlx::{Type, postgres::*};

    use super::Amount;

    impl Type<Postgres> for Amount {
        fn type_info() -> PgTypeInfo {
            <sqlx::types::Json<Amount> as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <sqlx::types::Json<Amount> as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for Amount {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <sqlx::types::Json<&Amount> as sqlx::Encode<'_, Postgres>>::encode(
                sqlx::types::Json(self),
                buf,
            )
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for Amount {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let json = <sqlx::types::Json<Amount> as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(json.0)
        }
    }

    impl PgHasArrayType for Amount {
        fn array_type_info() -> PgTypeInfo {
            <sqlx::types::Json<Amount> as PgHasArrayType>::array_type_info()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Btc, Satoshis, Usd, UsdCents};

    #[test]
    fn construction_captures_currency_metadata() {
        let amt = Amount::from(UsdCents::from(1000u64));
        assert_eq!(amt.currency(), CurrencyCode::USD);
        assert!(!amt.is_zero());

        let amt = Amount::from(Satoshis::from(50_000_000u64));
        assert_eq!(amt.currency(), CurrencyCode::BTC);
    }

    #[test]
    fn downcast_to_correct_type_succeeds() {
        let amt = Amount::from(UsdCents::from(1000u64));
        assert_eq!(amt.to_minor_units::<Usd>(), Some(UsdCents::from(1000u64)));
    }

    #[test]
    fn downcast_to_wrong_type_returns_none() {
        let amt = Amount::from(UsdCents::from(1000u64));
        assert_eq!(amt.to_minor_units::<Btc>(), None);
    }

    #[test]
    fn to_major_uses_correct_scale() {
        let usd = Amount::from(UsdCents::from(150u64)); // 150 cents = $1.50
        assert_eq!(usd.to_major(), Decimal::new(150, 2));

        let btc = Amount::from(Satoshis::from(100_000_000u64)); // 1 BTC
        assert_eq!(btc.to_major(), Decimal::from(1));
    }

    #[test]
    fn zero_check() {
        assert!(Amount::from(UsdCents::from(0u64)).is_zero());
        assert!(!Amount::from(UsdCents::from(1u64)).is_zero());
    }

    #[test]
    fn serde_roundtrip() {
        let amt = Amount::from(UsdCents::from(42u64));
        let json = serde_json::to_string(&amt).unwrap();
        let back: Amount = serde_json::from_str(&json).unwrap();
        assert_eq!(amt, back);
        assert_eq!(back.to_minor_units::<Usd>(), Some(UsdCents::from(42u64)));
    }

    // -- Amounts tests --

    #[test]
    fn amounts_typed_insert_and_get() {
        let mut amounts = Amounts::new([CurrencyCode::USD, CurrencyCode::BTC]);

        amounts.insert::<Usd>(UsdCents::from(500u64)).unwrap();
        amounts.insert::<Btc>(Satoshis::from(1_000_000u64)).unwrap();

        assert_eq!(amounts.get::<Usd>().unwrap(), Some(UsdCents::from(500u64)));
        assert_eq!(
            amounts.get::<Btc>().unwrap(),
            Some(Satoshis::from(1_000_000u64))
        );
    }

    #[test]
    fn amounts_rejects_disallowed_currency() {
        let mut amounts = Amounts::new([CurrencyCode::USD]);

        assert!(amounts.insert::<Btc>(Satoshis::from(100u64)).is_err());
        assert!(amounts.get::<Btc>().is_err());
    }

    #[test]
    fn amounts_is_zero() {
        let mut amounts = Amounts::new([CurrencyCode::USD, CurrencyCode::BTC]);
        assert!(amounts.is_zero());

        amounts.insert::<Usd>(UsdCents::from(0u64)).unwrap();
        assert!(amounts.is_zero());

        amounts.insert::<Btc>(Satoshis::from(1u64)).unwrap();
        assert!(!amounts.is_zero());
    }

    #[test]
    fn amounts_get_by_code() {
        let mut amounts = Amounts::new([CurrencyCode::USD]);
        amounts.insert::<Usd>(UsdCents::from(42u64)).unwrap();

        let amt = amounts.get_by_code(&CurrencyCode::USD).unwrap().unwrap();
        assert_eq!(amt.currency(), CurrencyCode::USD);
        assert_eq!(amt.to_minor_units::<Usd>(), Some(UsdCents::from(42u64)));
    }
}
