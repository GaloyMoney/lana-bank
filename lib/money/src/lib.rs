#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod code;
mod error;
mod map;

use std::{fmt, marker::PhantomData};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

pub use code::*;
pub use error::ConversionError;
pub use map::*;

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
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
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

// ---------------------------------------------------------------------------
// MinorUnits<C> — unsigned
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinorUnits<C: Currency> {
    value: u64,
    meta: C::Meta,
    _phantom: PhantomData<C>,
}

// --- Methods available on ALL MinorUnits<C> ---

impl<C: Currency> MinorUnits<C> {
    pub fn to_major(self) -> Decimal {
        Decimal::from(self.value) / Decimal::from(C::minor_units_per_major(&self.meta))
    }

    pub fn into_inner(self) -> u64 {
        self.value
    }

    pub fn is_zero(self) -> bool {
        self.value == 0
    }

    pub fn currency(&self) -> CurrencyCode {
        C::code(&self.meta)
    }

    pub fn to_untyped(self) -> MinorUnits<Untyped> {
        MinorUnits {
            value: self.value,
            meta: CurrencyMeta {
                code: C::code(&self.meta),
                minor_units_per_major: C::minor_units_per_major(&self.meta),
            },
            _phantom: PhantomData,
        }
    }
}

// --- Methods only for StaticCurrency (Usd, Btc, etc.) ---

impl<C: StaticCurrency> MinorUnits<C> {
    pub const ZERO: Self = Self {
        value: 0,
        meta: (),
        _phantom: PhantomData,
    };
    pub const ONE: Self = Self {
        value: 1,
        meta: (),
        _phantom: PhantomData,
    };

    pub fn try_from_major(major: Decimal) -> Result<Self, ConversionError> {
        let minor = major * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        if minor.trunc() != minor {
            return Err(ConversionError::PrecisionLoss(major));
        }
        if minor < Decimal::new(0, 0) {
            return Err(ConversionError::UnexpectedNegativeNumber(minor));
        }
        Ok(Self {
            value: u64::try_from(minor)?,
            meta: (),
            _phantom: PhantomData,
        })
    }
}

// --- Currency-specific convenience methods ---

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

// --- UntypedAmount-specific methods ---

impl MinorUnits<Untyped> {
    pub fn try_from_major(currency: CurrencyCode, major: Decimal) -> Result<Self, ConversionError> {
        match currency {
            CurrencyCode::USD => Ok(MinorUnits::<Usd>::try_from_major(major)?.into()),
            CurrencyCode::BTC => Ok(MinorUnits::<Btc>::try_from_major(major)?.into()),
            _ => Err(ConversionError::UnsupportedCurrency(currency)),
        }
    }

    pub fn to_typed<C: StaticCurrency>(&self) -> Option<MinorUnits<C>> {
        (self.meta.code == C::CODE).then(|| MinorUnits {
            value: self.value,
            meta: (),
            _phantom: PhantomData,
        })
    }
}

// --- Serde for StaticCurrency: bare u64 (backwards compatible) ---

impl<C: StaticCurrency> Serialize for MinorUnits<C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de, C: StaticCurrency> Deserialize<'de> for MinorUnits<C> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u64::deserialize(deserializer).map(|v| Self {
            value: v,
            meta: (),
            _phantom: PhantomData,
        })
    }
}

// --- Serde for Untyped: struct with currency metadata ---

impl Serialize for MinorUnits<Untyped> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Amount", 3)?;
        s.serialize_field("currency", &self.meta.code)?;
        s.serialize_field("minor_units", &self.value)?;
        s.serialize_field("minor_units_per_major", &self.meta.minor_units_per_major)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for MinorUnits<Untyped> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            currency: CurrencyCode,
            minor_units: u64,
            minor_units_per_major: u64,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            value: raw.minor_units,
            meta: CurrencyMeta {
                code: raw.currency,
                minor_units_per_major: raw.minor_units_per_major,
            },
            _phantom: PhantomData,
        })
    }
}

// --- JsonSchema ---

#[cfg(feature = "json-schema")]
impl<C: StaticCurrency> JsonSchema for MinorUnits<C> {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        u64::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        u64::json_schema(generator)
    }
}

#[cfg(feature = "json-schema")]
impl JsonSchema for MinorUnits<Untyped> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Amount")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        #[derive(JsonSchema)]
        #[allow(dead_code)]
        struct AmountSchema {
            currency: CurrencyCode,
            minor_units: u64,
            minor_units_per_major: u64,
        }
        AmountSchema::json_schema(generator)
    }
}

// --- Display ---

impl<C: Currency> fmt::Display for MinorUnits<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// --- Default (only for StaticCurrency) ---

impl<C: StaticCurrency> Default for MinorUnits<C> {
    fn default() -> Self {
        Self::ZERO
    }
}

// --- Arithmetic (only for StaticCurrency — prevents cross-currency addition) ---

impl<C: StaticCurrency> std::ops::Add for MinorUnits<C> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
            meta: (),
            _phantom: PhantomData,
        }
    }
}

impl<C: StaticCurrency> std::ops::Sub for MinorUnits<C> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            value: self.value - other.value,
            meta: (),
            _phantom: PhantomData,
        }
    }
}

impl<C: StaticCurrency> std::ops::AddAssign for MinorUnits<C> {
    fn add_assign(&mut self, other: Self) {
        self.value += other.value;
    }
}

impl<C: StaticCurrency> std::ops::SubAssign for MinorUnits<C> {
    fn sub_assign(&mut self, other: Self) {
        self.value -= other.value;
    }
}

impl std::ops::Mul<u64> for MinorUnits<Usd> {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self {
        Self {
            value: self.value * rhs,
            meta: (),
            _phantom: PhantomData,
        }
    }
}

// --- From<u64> (only for StaticCurrency) ---

impl<C: StaticCurrency> From<u64> for MinorUnits<C> {
    fn from(value: u64) -> Self {
        Self {
            value,
            meta: (),
            _phantom: PhantomData,
        }
    }
}

// --- Typed → Untyped (always succeeds) ---

impl<C: StaticCurrency> From<MinorUnits<C>> for MinorUnits<Untyped> {
    fn from(typed: MinorUnits<C>) -> Self {
        Self {
            value: typed.value,
            meta: CurrencyMeta::of::<C>(),
            _phantom: PhantomData,
        }
    }
}

// --- SignedMinorUnits → MinorUnits conversion ---

impl<C: StaticCurrency> TryFrom<SignedMinorUnits<C>> for MinorUnits<C> {
    type Error = ConversionError;
    fn try_from(value: SignedMinorUnits<C>) -> Result<Self, Self::Error> {
        let major = value.to_major();
        Self::try_from_major(major)
    }
}

// ---------------------------------------------------------------------------
// SignedMinorUnits<C> — signed (only for StaticCurrency)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignedMinorUnits<C: StaticCurrency>(i64, PhantomData<C>);

impl<C: StaticCurrency> SignedMinorUnits<C> {
    pub const ZERO: Self = Self(0, PhantomData);
    pub const ONE: Self = Self(1, PhantomData);

    pub fn to_major(self) -> Decimal {
        Decimal::from(self.0) / Decimal::from(C::MINOR_UNITS_PER_MAJOR)
    }

    pub fn try_from_major(major: Decimal) -> Result<Self, ConversionError> {
        let minor = major * Decimal::from(C::MINOR_UNITS_PER_MAJOR);
        if minor.trunc() != minor {
            return Err(ConversionError::PrecisionLoss(major));
        }
        Ok(Self(
            i64::try_from(minor).map_err(|_| ConversionError::Overflow)?,
            PhantomData,
        ))
    }

    pub fn checked_abs(self) -> Result<Self, ConversionError> {
        self.0
            .checked_abs()
            .map(|v| Self(v, PhantomData))
            .ok_or(ConversionError::Overflow)
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
        Self::try_from_major(btc).expect("BTC must convert to whole satoshis")
    }
}

impl SignedMinorUnits<Usd> {
    pub fn to_usd(self) -> Decimal {
        self.to_major()
    }

    pub fn from_usd(usd: Decimal) -> Self {
        Self::try_from_major(usd).expect("USD must convert to whole cents")
    }
}

// --- Serde ---

impl<C: StaticCurrency> Serialize for SignedMinorUnits<C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, C: StaticCurrency> Deserialize<'de> for SignedMinorUnits<C> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i64::deserialize(deserializer).map(|v| Self(v, PhantomData))
    }
}

// --- JsonSchema ---

#[cfg(feature = "json-schema")]
impl<C: StaticCurrency> JsonSchema for SignedMinorUnits<C> {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        i64::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        i64::json_schema(generator)
    }
}

// --- Display / Default ---

impl<C: StaticCurrency> fmt::Display for SignedMinorUnits<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<C: StaticCurrency> Default for SignedMinorUnits<C> {
    fn default() -> Self {
        Self::ZERO
    }
}

// --- Arithmetic ---

impl<C: StaticCurrency> std::ops::Add for SignedMinorUnits<C> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, PhantomData)
    }
}

impl<C: StaticCurrency> std::ops::Sub for SignedMinorUnits<C> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, PhantomData)
    }
}

// --- From conversions ---

impl<C: StaticCurrency> TryFrom<MinorUnits<C>> for SignedMinorUnits<C> {
    type Error = ConversionError;
    fn try_from(val: MinorUnits<C>) -> Result<Self, Self::Error> {
        Ok(Self(
            i64::try_from(val.value).map_err(|_| ConversionError::Overflow)?,
            PhantomData,
        ))
    }
}

// ---------------------------------------------------------------------------
// SQLx impls
// ---------------------------------------------------------------------------

#[cfg(feature = "sqlx")]
mod minor_units_sqlx {
    use sqlx::{Type, postgres::*};

    use super::*;

    // --- StaticCurrency: stored as BIGINT ---

    impl<C: StaticCurrency> Type<Postgres> for MinorUnits<C> {
        fn type_info() -> PgTypeInfo {
            <i64 as Type<Postgres>>::type_info()
        }
        fn compatible(ty: &PgTypeInfo) -> bool {
            <i64 as Type<Postgres>>::compatible(ty)
        }
    }

    impl<C: StaticCurrency> sqlx::Encode<'_, Postgres> for MinorUnits<C> {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            let val = i64::try_from(self.into_inner())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>)?;
            <i64 as sqlx::Encode<'_, Postgres>>::encode(val, buf)
        }
    }

    impl<'r, C: StaticCurrency> sqlx::Decode<'r, Postgres> for MinorUnits<C> {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let val = <i64 as sqlx::Decode<Postgres>>::decode(value)?;
            let val = u64::try_from(val)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>)?;
            Ok(MinorUnits::from(val))
        }
    }

    impl<C: StaticCurrency> PgHasArrayType for MinorUnits<C> {
        fn array_type_info() -> PgTypeInfo {
            <i64 as PgHasArrayType>::array_type_info()
        }
    }

    // --- Untyped: stored as JSONB ---

    impl Type<Postgres> for MinorUnits<Untyped> {
        fn type_info() -> PgTypeInfo {
            <sqlx::types::Json<MinorUnits<Untyped>> as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <sqlx::types::Json<MinorUnits<Untyped>> as Type<Postgres>>::compatible(ty)
        }
    }

    impl sqlx::Encode<'_, Postgres> for MinorUnits<Untyped> {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <sqlx::types::Json<&MinorUnits<Untyped>> as sqlx::Encode<'_, Postgres>>::encode(
                sqlx::types::Json(self),
                buf,
            )
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for MinorUnits<Untyped> {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let json =
                <sqlx::types::Json<MinorUnits<Untyped>> as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(json.0)
        }
    }

    impl PgHasArrayType for MinorUnits<Untyped> {
        fn array_type_info() -> PgTypeInfo {
            <sqlx::types::Json<MinorUnits<Untyped>> as PgHasArrayType>::array_type_info()
        }
    }
}

#[cfg(feature = "sqlx")]
mod signed_minor_units_sqlx {
    use sqlx::{Type, postgres::*};

    use super::*;

    impl<C: StaticCurrency> Type<Postgres> for SignedMinorUnits<C> {
        fn type_info() -> PgTypeInfo {
            <i64 as Type<Postgres>>::type_info()
        }
        fn compatible(ty: &PgTypeInfo) -> bool {
            <i64 as Type<Postgres>>::compatible(ty)
        }
    }

    impl<C: StaticCurrency> sqlx::Encode<'_, Postgres> for SignedMinorUnits<C> {
        fn encode_by_ref(
            &self,
            buf: &mut PgArgumentBuffer,
        ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <i64 as sqlx::Encode<'_, Postgres>>::encode(self.into_inner(), buf)
        }
    }

    impl<'r, C: StaticCurrency> sqlx::Decode<'r, Postgres> for SignedMinorUnits<C> {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let val = <i64 as sqlx::Decode<Postgres>>::decode(value)?;
            Ok(SignedMinorUnits(val, std::marker::PhantomData))
        }
    }

    impl<C: StaticCurrency> PgHasArrayType for SignedMinorUnits<C> {
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
// Type aliases
// ---------------------------------------------------------------------------

pub type UsdCents = MinorUnits<Usd>;
pub type Satoshis = MinorUnits<Btc>;
pub type SignedUsdCents = SignedMinorUnits<Usd>;
pub type SignedSatoshis = SignedMinorUnits<Btc>;
pub type UntypedAmount = MinorUnits<Untyped>;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_currency_zero_and_one() {
        assert!(UsdCents::ZERO.is_zero());
        assert!(!UsdCents::ONE.is_zero());
        assert!(Satoshis::ZERO.is_zero());
    }

    #[test]
    fn static_currency_from_u64() {
        let cents = UsdCents::from(1500u64);
        assert_eq!(cents.into_inner(), 1500);
        assert_eq!(cents.currency(), CurrencyCode::USD);
    }

    #[test]
    fn to_major_works_for_static() {
        let cents = UsdCents::from(150u64);
        assert_eq!(cents.to_major(), Decimal::new(150, 2)); // $1.50

        let sats = Satoshis::from(100_000_000u64);
        assert_eq!(sats.to_major(), Decimal::from(1)); // 1 BTC
    }

    #[test]
    fn typed_to_untyped_conversion() {
        let cents = UsdCents::from(1500u64);
        let untyped: UntypedAmount = cents.into();

        assert_eq!(untyped.currency(), CurrencyCode::USD);
        assert_eq!(untyped.into_inner(), 1500);
        assert_eq!(untyped.to_major(), Decimal::new(1500, 2)); // $15.00
    }

    #[test]
    fn untyped_to_typed_downcast() {
        let cents = UsdCents::from(1500u64);
        let untyped: UntypedAmount = cents.into();

        // Correct downcast
        let back = untyped.to_typed::<Usd>().unwrap();
        assert_eq!(back, UsdCents::from(1500u64));

        // Wrong downcast
        assert!(untyped.to_typed::<Btc>().is_none());
    }

    #[test]
    fn untyped_try_from_major() {
        let amt = UntypedAmount::try_from_major(CurrencyCode::USD, Decimal::new(1500, 2)).unwrap();
        assert_eq!(amt.currency(), CurrencyCode::USD);
        assert_eq!(amt.into_inner(), 1500);

        let amt = UntypedAmount::try_from_major(CurrencyCode::BTC, Decimal::from(1)).unwrap();
        assert_eq!(amt.currency(), CurrencyCode::BTC);
        assert_eq!(amt.into_inner(), 100_000_000);
    }

    #[test]
    fn static_serde_is_bare_u64() {
        let cents = UsdCents::from(42u64);
        let json = serde_json::to_string(&cents).unwrap();
        assert_eq!(json, "42");

        let back: UsdCents = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cents);
    }

    #[test]
    fn untyped_serde_is_struct() {
        let cents = UsdCents::from(42u64);
        let untyped: UntypedAmount = cents.into();
        let json = serde_json::to_string(&untyped).unwrap();
        assert!(json.contains("\"currency\""));
        assert!(json.contains("\"minor_units\""));
        assert!(json.contains("\"minor_units_per_major\""));

        let back: UntypedAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(back, untyped);
        assert_eq!(back.to_typed::<Usd>(), Some(UsdCents::from(42u64)));
    }

    #[test]
    fn arithmetic_works_for_static() {
        let a = UsdCents::from(100u64);
        let b = UsdCents::from(50u64);
        assert_eq!((a + b).into_inner(), 150);
        assert_eq!((a - b).into_inner(), 50);
    }

    // Arithmetic does NOT compile for Untyped — this is intentional.
    // let a: UntypedAmount = ...; let b: UntypedAmount = ...; a + b; // ERROR

    #[test]
    fn size_of_types() {
        // StaticCurrency: just u64 + ZSTs = 8 bytes
        assert_eq!(std::mem::size_of::<UsdCents>(), 8);
        assert_eq!(std::mem::size_of::<Satoshis>(), 8);

        // Untyped: u64 + CurrencyMeta (CurrencyCode + u64) + ZST
        // CurrencyCode is &'static str = pointer = 16 bytes (ptr + len)
        let untyped_size = std::mem::size_of::<UntypedAmount>();
        assert!(
            untyped_size > 8,
            "UntypedAmount should be larger: {untyped_size}"
        );
    }
}
