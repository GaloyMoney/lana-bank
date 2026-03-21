use std::fmt;

use serde::{Deserialize, Serialize};

use crate::CurrencyCode;

// ---------------------------------------------------------------------------
// Currency trait — associated type Meta controls runtime storage
// ---------------------------------------------------------------------------

pub trait Currency:
    'static + Copy + Clone + Send + Sync + fmt::Debug + PartialEq + Eq + std::hash::Hash
{
    type Meta: Copy + Clone + fmt::Debug + PartialEq + Eq + std::hash::Hash + Send + Sync;

    fn code(meta: &Self::Meta) -> CurrencyCode;
    fn minor_units_per_major(meta: &Self::Meta) -> u64;
}

/// Marker subtrait for currencies whose metadata is fully known at compile
/// time (`Meta = ()`).  All construction helpers (`From<u64>`, `ZERO`, `ONE`,
/// arithmetic, serde-as-u64) are gated on this bound so they never apply to
/// `Untyped`.
pub trait StaticCurrency: Currency<Meta = ()> {
    const CODE: CurrencyCode;
    const MINOR_UNITS_PER_MAJOR: u64;
}

// ---------------------------------------------------------------------------
// Static currency markers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Usd;

impl Currency for Usd {
    type Meta = ();
    fn code(_: &()) -> CurrencyCode {
        CurrencyCode::USD
    }
    fn minor_units_per_major(_: &()) -> u64 {
        100
    }
}

impl StaticCurrency for Usd {
    const CODE: CurrencyCode = CurrencyCode::USD;
    const MINOR_UNITS_PER_MAJOR: u64 = 100;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Btc;

impl Currency for Btc {
    type Meta = ();
    fn code(_: &()) -> CurrencyCode {
        CurrencyCode::BTC
    }
    fn minor_units_per_major(_: &()) -> u64 {
        100_000_000
    }
}

impl StaticCurrency for Btc {
    const CODE: CurrencyCode = CurrencyCode::BTC;
    const MINOR_UNITS_PER_MAJOR: u64 = 100_000_000;
}

// ---------------------------------------------------------------------------
// Untyped currency marker — carries metadata at runtime
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct CurrencyMeta {
    pub code: CurrencyCode,
    pub minor_units_per_major: u64,
}

impl CurrencyMeta {
    pub fn of<C: StaticCurrency>() -> Self {
        Self {
            code: C::CODE,
            minor_units_per_major: C::MINOR_UNITS_PER_MAJOR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Untyped;

impl Currency for Untyped {
    type Meta = CurrencyMeta;
    fn code(meta: &CurrencyMeta) -> CurrencyCode {
        meta.code
    }
    fn minor_units_per_major(meta: &CurrencyMeta) -> u64 {
        meta.minor_units_per_major
    }
}
