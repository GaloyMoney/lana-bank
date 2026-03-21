use std::fmt;

use serde::{Deserialize, Serialize};

use crate::CurrencyCode;

pub trait Currency:
    'static + Copy + Clone + Send + Sync + fmt::Debug + PartialEq + Eq + std::hash::Hash
{
    fn code(&self) -> CurrencyCode;
    fn minor_units_per_major(&self) -> u64;
}

/// Subtrait for currencies fully known at compile time.
///
/// Static currencies are unit structs (e.g. `struct Usd;`) — zero-sized types
/// with exactly one possible value.  Rust's type system treats generic type
/// parameters as opaque, so `INSTANCE` provides that value for generic code
/// that needs to construct `MinorUnits<C>` (including const contexts like
/// `ZERO`/`ONE` where trait methods like `Default::default()` cannot be called).
pub trait StaticCurrency: Currency {
    const CODE: CurrencyCode;
    const MINOR_UNITS_PER_MAJOR: u64;
    const INSTANCE: Self;
}

macro_rules! define_static_currency {
    ($Name:ident, $code:expr, $minor_units_per_major:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $Name;

        impl Currency for $Name {
            fn code(&self) -> CurrencyCode {
                $code
            }
            fn minor_units_per_major(&self) -> u64 {
                $minor_units_per_major
            }
        }

        impl StaticCurrency for $Name {
            const CODE: CurrencyCode = $code;
            const MINOR_UNITS_PER_MAJOR: u64 = $minor_units_per_major;
            const INSTANCE: Self = $Name;
        }
    };
}

define_static_currency!(Usd, CurrencyCode::USD, 100);
define_static_currency!(Btc, CurrencyCode::BTC, 100_000_000);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Untyped {
    code: CurrencyCode,
    minor_units_per_major: u64,
}

impl Untyped {
    pub(crate) fn from_raw(code: CurrencyCode, minor_units_per_major: u64) -> Self {
        Self {
            code,
            minor_units_per_major,
        }
    }

    pub(crate) fn of<C: StaticCurrency>() -> Self {
        Self::from_raw(C::CODE, C::MINOR_UNITS_PER_MAJOR)
    }
}

impl Currency for Untyped {
    fn code(&self) -> CurrencyCode {
        self.code
    }
    fn minor_units_per_major(&self) -> u64 {
        self.minor_units_per_major
    }
}
