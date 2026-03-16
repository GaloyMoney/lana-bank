use std::fmt;
use std::marker::PhantomData;

use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};

use money::{Btc, Currency, MinorUnits, Usd};

/// Computes the number of decimal places in major units for a currency.
///
/// E.g., for USD (100 cents/dollar) returns 2, for BTC (100M sats/BTC) returns 8.
fn minor_unit_decimal_places<C: Currency>() -> u32 {
    let mut n = C::MINOR_UNITS_PER_MAJOR;
    let mut dp = 0u32;
    while n > 1 {
        n /= 10;
        dp += 1;
    }
    dp
}

/// An exchange rate expressing the price of 1 major unit of `F` ("from"
/// currency) in minor units of `T` ("to" currency).
///
/// For example, `ExchangeRate<Btc, Usd>` stores the USD-cent price of 1 BTC.
/// The backward-compatible alias [`PriceOfOneBTC`] is provided for this pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExchangeRate<F: Currency, T: Currency> {
    inner: MinorUnits<T>,
    _from: PhantomData<F>,
}

impl<F: Currency, T: Currency> ExchangeRate<F, T> {
    pub const ZERO: Self = Self {
        inner: MinorUnits::<T>::ZERO,
        _from: PhantomData,
    };

    pub const fn new(price: MinorUnits<T>) -> Self {
        Self {
            inner: price,
            _from: PhantomData,
        }
    }

    pub fn into_inner(self) -> MinorUnits<T> {
        self.inner
    }

    /// Convert an amount in the `From` currency to the `To` currency.
    ///
    /// The caller chooses the [`RoundingStrategy`] appropriate for their
    /// domain (e.g. `ToZero` for conservative value estimates).
    pub fn convert(self, amount: MinorUnits<F>, strategy: RoundingStrategy) -> MinorUnits<T> {
        let to_major = amount.to_major() * self.inner.to_major();
        let dp = minor_unit_decimal_places::<T>();
        let rounded = to_major.round_dp_with_strategy(dp, strategy);
        MinorUnits::<T>::try_from_major(rounded)
            .expect("Rounded decimal should have no fractional component")
    }

    /// Convert an amount in the `To` currency back to the `From` currency.
    ///
    /// The caller chooses the [`RoundingStrategy`] appropriate for their
    /// domain (e.g. `AwayFromZero` for conservative collateral requirements).
    pub fn reverse_convert(
        self,
        amount: MinorUnits<T>,
        strategy: RoundingStrategy,
    ) -> MinorUnits<F> {
        let from_major = amount.to_major() / self.inner.to_major();
        let dp = minor_unit_decimal_places::<F>();
        let rounded = from_major.round_dp_with_strategy(dp, strategy);
        MinorUnits::<F>::try_from_major(rounded)
            .expect("Rounded decimal should have no fractional component")
    }
}

// --- BTC/USD-specific convenience methods (backward compat) ---

impl ExchangeRate<Btc, Usd> {
    pub fn cents_to_sats_round_up(self, cents: MinorUnits<Usd>) -> MinorUnits<Btc> {
        self.reverse_convert(cents, RoundingStrategy::AwayFromZero)
    }

    pub fn sats_to_cents_round_down(self, sats: MinorUnits<Btc>) -> MinorUnits<Usd> {
        self.convert(sats, RoundingStrategy::ToZero)
    }
}

// --- Serde (serialize as inner value for backward compat) ---

impl<F: Currency, T: Currency> Serialize for ExchangeRate<F, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, F: Currency, T: Currency> Deserialize<'de> for ExchangeRate<F, T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        MinorUnits::<T>::deserialize(deserializer).map(|inner| Self {
            inner,
            _from: PhantomData,
        })
    }
}

// --- Display ---

impl<F: Currency, T: Currency> fmt::Display for ExchangeRate<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dp = minor_unit_decimal_places::<T>() as usize;
        write!(f, "{:.dp$}", self.inner.to_major())
    }
}

// --- JsonSchema (backward-compat name for BTC/USD) ---

#[cfg(feature = "json-schema")]
impl schemars::JsonSchema for ExchangeRate<Btc, Usd> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "PriceOfOneBTC".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        u64::json_schema(generator)
    }
}

// --- Backward-compatible type alias ---

pub type PriceOfOneBTC = ExchangeRate<Btc, Usd>;
