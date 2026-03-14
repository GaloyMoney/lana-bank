#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use std::{borrow::Cow, fmt, marker::PhantomData};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// ---------------------------------------------------------------------------
// Currency trait + marker types
// ---------------------------------------------------------------------------

pub trait Currency:
    'static + Copy + Clone + Send + Sync + fmt::Debug + PartialEq + Eq + std::hash::Hash
{
    const CODE: &'static str;
    const MINOR_UNITS_PER_MAJOR: u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Usd;

impl Currency for Usd {
    const CODE: &'static str = "USD";
    const MINOR_UNITS_PER_MAJOR: u64 = 100;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Btc;

impl Currency for Btc {
    const CODE: &'static str = "BTC";
    const MINOR_UNITS_PER_MAJOR: u64 = 100_000_000;
}

// ---------------------------------------------------------------------------
// Constants (backward compat)
// ---------------------------------------------------------------------------

pub const SATS_PER_BTC: Decimal = dec!(100_000_000);
pub const CENTS_PER_USD: Decimal = dec!(100);

// ---------------------------------------------------------------------------
// ConversionError
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("ConversionError - DecimalError: {0}")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("ConversionError - UnexpectedNegativeNumber: {0}")]
    UnexpectedNegativeNumber(rust_decimal::Decimal),
    #[error("ConversionError - Overflow")]
    Overflow,
}

impl ErrorSeverity for ConversionError {
    fn severity(&self) -> Level {
        match self {
            Self::DecimalError(_) => Level::ERROR,
            Self::UnexpectedNegativeNumber(_) => Level::WARN,
            Self::Overflow => Level::ERROR,
        }
    }
}

// ---------------------------------------------------------------------------
// MinorUnits<C> — unsigned
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinorUnits<C: Currency>(u64, PhantomData<C>);

impl<C: Currency> MinorUnits<C> {
    pub const ZERO: Self = Self(0, PhantomData);
    pub const ONE: Self = Self(1, PhantomData);

    pub fn to_major(self) -> Decimal {
        Decimal::from(self.0) / Decimal::from(C::MINOR_UNITS_PER_MAJOR)
    }

    pub fn try_from_major(major: Decimal) -> Result<Self, ConversionError> {
        let minor = major * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        assert!(minor.trunc() == minor, "Minor units must be an integer");
        if minor < Decimal::new(0, 0) {
            return Err(ConversionError::UnexpectedNegativeNumber(minor));
        }
        Ok(Self(u64::try_from(minor)?, PhantomData))
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

// --- Currency-specific methods ---

impl MinorUnits<Btc> {
    pub fn to_btc(self) -> Decimal {
        self.to_major()
    }

    pub fn try_from_btc(btc: Decimal) -> Result<Self, ConversionError> {
        Self::try_from_major(btc)
    }

    pub fn formatted_btc(self) -> String {
        format!("{:.8}", self.to_btc())
    }
}

impl MinorUnits<Usd> {
    pub fn to_usd(self) -> Decimal {
        self.to_major()
    }

    pub fn try_from_usd(usd: Decimal) -> Result<Self, ConversionError> {
        Self::try_from_major(usd)
    }

    pub fn formatted_usd(self) -> String {
        format!("${:.2}", self.to_usd())
    }
}

// --- Serde ---

impl<C: Currency> Serialize for MinorUnits<C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, C: Currency> Deserialize<'de> for MinorUnits<C> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u64::deserialize(deserializer).map(|v| Self(v, PhantomData))
    }
}

// --- JsonSchema ---

#[cfg(feature = "json-schema")]
impl<C: Currency> JsonSchema for MinorUnits<C> {
    fn schema_name() -> Cow<'static, str> {
        format!("MinorUnits_{}", C::CODE).into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        u64::json_schema(generator)
    }
}

// --- Display / Default ---

impl<C: Currency> fmt::Display for MinorUnits<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<C: Currency> Default for MinorUnits<C> {
    fn default() -> Self {
        Self::ZERO
    }
}

// --- Arithmetic ---

impl<C: Currency> std::ops::Add for MinorUnits<C> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, PhantomData)
    }
}

impl<C: Currency> std::ops::Sub for MinorUnits<C> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, PhantomData)
    }
}

impl<C: Currency> std::ops::AddAssign for MinorUnits<C> {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl<C: Currency> std::ops::SubAssign for MinorUnits<C> {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl std::ops::Mul<u64> for MinorUnits<Usd> {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self {
        Self(self.0 * rhs, PhantomData)
    }
}

// --- From conversions ---

impl<C: Currency> From<u64> for MinorUnits<C> {
    fn from(value: u64) -> Self {
        Self(value, PhantomData)
    }
}

impl<C: Currency> TryFrom<SignedMinorUnits<C>> for MinorUnits<C> {
    type Error = ConversionError;
    fn try_from(value: SignedMinorUnits<C>) -> Result<Self, Self::Error> {
        let major = value.to_major();
        Self::try_from_major(major)
    }
}

// ---------------------------------------------------------------------------
// SignedMinorUnits<C> — signed
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignedMinorUnits<C: Currency>(i64, PhantomData<C>);

impl<C: Currency> SignedMinorUnits<C> {
    pub const ZERO: Self = Self(0, PhantomData);
    pub const ONE: Self = Self(1, PhantomData);

    pub fn to_major(self) -> Decimal {
        Decimal::from(self.0) / Decimal::from(C::MINOR_UNITS_PER_MAJOR)
    }

    pub fn from_major(major: Decimal) -> Self {
        let minor = major * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        assert!(minor.trunc() == minor, "Minor units must be an integer");
        Self(
            i64::try_from(minor).expect("Minor units must fit i64"),
            PhantomData,
        )
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs(), PhantomData)
    }

    pub fn into_inner(self) -> i64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

// --- Currency-specific methods ---

impl SignedMinorUnits<Btc> {
    pub fn to_btc(self) -> Decimal {
        self.to_major()
    }

    pub fn from_btc(btc: Decimal) -> Self {
        Self::from_major(btc)
    }
}

impl SignedMinorUnits<Usd> {
    pub fn to_usd(self) -> Decimal {
        self.to_major()
    }

    pub fn from_usd(usd: Decimal) -> Self {
        Self::from_major(usd)
    }
}

// --- Serde ---

impl<C: Currency> Serialize for SignedMinorUnits<C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, C: Currency> Deserialize<'de> for SignedMinorUnits<C> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i64::deserialize(deserializer).map(|v| Self(v, PhantomData))
    }
}

// --- JsonSchema ---

#[cfg(feature = "json-schema")]
impl<C: Currency> JsonSchema for SignedMinorUnits<C> {
    fn schema_name() -> Cow<'static, str> {
        format!("SignedMinorUnits_{}", C::CODE).into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        i64::json_schema(generator)
    }
}

// --- Display / Default ---

impl<C: Currency> fmt::Display for SignedMinorUnits<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<C: Currency> Default for SignedMinorUnits<C> {
    fn default() -> Self {
        Self::ZERO
    }
}

// --- Arithmetic ---

impl<C: Currency> std::ops::Add for SignedMinorUnits<C> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, PhantomData)
    }
}

impl<C: Currency> std::ops::Sub for SignedMinorUnits<C> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, PhantomData)
    }
}

// --- From conversions ---

impl<C: Currency> From<MinorUnits<C>> for SignedMinorUnits<C> {
    fn from(val: MinorUnits<C>) -> Self {
        Self(
            i64::try_from(val.0).expect("Minor units must fit i64"),
            PhantomData,
        )
    }
}

// ---------------------------------------------------------------------------
// SQLx impls
// ---------------------------------------------------------------------------

#[cfg(feature = "sqlx")]
mod minor_units_sqlx {
    use sqlx::{Type, postgres::*};

    use super::*;

    impl<C: Currency> Type<Postgres> for MinorUnits<C> {
        fn type_info() -> PgTypeInfo {
            <i64 as Type<Postgres>>::type_info()
        }
        fn compatible(ty: &PgTypeInfo) -> bool {
            <i64 as Type<Postgres>>::compatible(ty)
        }
    }

    impl<C: Currency> sqlx::Encode<'_, Postgres> for MinorUnits<C> {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            let val = i64::try_from(self.into_inner())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>)?;
            <i64 as sqlx::Encode<'_, Postgres>>::encode(val, buf)
        }
    }

    impl<'r, C: Currency> sqlx::Decode<'r, Postgres> for MinorUnits<C> {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let val = <i64 as sqlx::Decode<Postgres>>::decode(value)?;
            let val = u64::try_from(val)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>)?;
            Ok(MinorUnits::from(val))
        }
    }

    impl<C: Currency> PgHasArrayType for MinorUnits<C> {
        fn array_type_info() -> PgTypeInfo {
            <i64 as PgHasArrayType>::array_type_info()
        }
    }
}

// ---------------------------------------------------------------------------
// GraphQL scalars
// ---------------------------------------------------------------------------

#[cfg(feature = "graphql")]
mod graphql_scalars {
    use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

    use super::*;

    // We cannot use async_graphql::scalar!() on type aliases of generic types,
    // so we implement the Scalar trait manually for each concrete alias.

    #[Scalar(name = "Satoshis")]
    impl ScalarType for MinorUnits<Btc> {
        fn parse(value: Value) -> InputValueResult<Self> {
            match &value {
                Value::Number(n) => {
                    let v = n
                        .as_u64()
                        .ok_or_else(|| InputValueError::expected_type(value))?;
                    Ok(Self::from(v))
                }
                _ => Err(InputValueError::expected_type(value)),
            }
        }

        fn to_value(&self) -> Value {
            Value::Number(self.into_inner().into())
        }
    }

    #[Scalar(name = "UsdCents")]
    impl ScalarType for MinorUnits<Usd> {
        fn parse(value: Value) -> InputValueResult<Self> {
            match &value {
                Value::Number(n) => {
                    let v = n
                        .as_u64()
                        .ok_or_else(|| InputValueError::expected_type(value))?;
                    Ok(Self::from(v))
                }
                _ => Err(InputValueError::expected_type(value)),
            }
        }

        fn to_value(&self) -> Value {
            Value::Number(self.into_inner().into())
        }
    }

    #[Scalar(name = "SignedSatoshis")]
    impl ScalarType for SignedMinorUnits<Btc> {
        fn parse(value: Value) -> InputValueResult<Self> {
            match &value {
                Value::Number(n) => {
                    let v = n
                        .as_i64()
                        .ok_or_else(|| InputValueError::expected_type(value))?;
                    Ok(Self(v, PhantomData))
                }
                _ => Err(InputValueError::expected_type(value)),
            }
        }

        fn to_value(&self) -> Value {
            Value::Number(self.into_inner().into())
        }
    }

    #[Scalar(name = "SignedUsdCents")]
    impl ScalarType for SignedMinorUnits<Usd> {
        fn parse(value: Value) -> InputValueResult<Self> {
            match &value {
                Value::Number(n) => {
                    let v = n
                        .as_i64()
                        .ok_or_else(|| InputValueError::expected_type(value))?;
                    Ok(Self(v, PhantomData))
                }
                _ => Err(InputValueError::expected_type(value)),
            }
        }

        fn to_value(&self) -> Value {
            Value::Number(self.into_inner().into())
        }
    }
}

// ---------------------------------------------------------------------------
// Type aliases — backward-compatible public API
// ---------------------------------------------------------------------------

pub type UsdCents = MinorUnits<Usd>;
pub type Satoshis = MinorUnits<Btc>;
pub type SignedUsdCents = SignedMinorUnits<Usd>;
pub type SignedSatoshis = SignedMinorUnits<Btc>;
