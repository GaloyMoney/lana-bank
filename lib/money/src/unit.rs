use std::{fmt, marker::PhantomData};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{Btc, ConversionError, Currency, CurrencyCode, StaticCurrency, Untyped, Usd};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinorUnits<C: Currency> {
    value: u64,
    currency: C,
}

impl<C: Currency> MinorUnits<C> {
    pub fn to_major(self) -> Decimal {
        Decimal::from(self.value) / Decimal::from(self.currency.minor_units_per_major())
    }

    pub fn into_inner(self) -> u64 {
        self.value
    }

    pub fn is_zero(self) -> bool {
        self.value == 0
    }

    pub fn currency(&self) -> CurrencyCode {
        self.currency.code()
    }
}

impl<C: Currency> fmt::Display for MinorUnits<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<C: StaticCurrency> Default for MinorUnits<C> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<C: StaticCurrency> MinorUnits<C> {
    pub const ZERO: Self = Self {
        value: 0,
        currency: C::INSTANCE,
    };
    pub const ONE: Self = Self {
        value: 1,
        currency: C::INSTANCE,
    };

    pub fn to_untyped(self) -> MinorUnits<Untyped> {
        MinorUnits {
            value: self.value,
            currency: Untyped::of::<C>(),
        }
    }

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
            currency: C::INSTANCE,
        })
    }
}

impl<C: StaticCurrency> From<u64> for MinorUnits<C> {
    fn from(value: u64) -> Self {
        Self {
            value,
            currency: C::INSTANCE,
        }
    }
}

impl<C: StaticCurrency> std::ops::Add for MinorUnits<C> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
            currency: C::INSTANCE,
        }
    }
}

impl<C: StaticCurrency> std::ops::Sub for MinorUnits<C> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            value: self.value - other.value,
            currency: C::INSTANCE,
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

impl<C: StaticCurrency> Serialize for MinorUnits<C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de, C: StaticCurrency> Deserialize<'de> for MinorUnits<C> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u64::deserialize(deserializer).map(|v| Self {
            value: v,
            currency: C::INSTANCE,
        })
    }
}

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

impl<C: StaticCurrency> From<MinorUnits<C>> for MinorUnits<Untyped> {
    fn from(typed: MinorUnits<C>) -> Self {
        Self {
            value: typed.value,
            currency: Untyped::of::<C>(),
        }
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

impl std::ops::Mul<u64> for MinorUnits<Usd> {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self {
        Self {
            value: self.value * rhs,
            currency: Usd,
        }
    }
}

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

impl MinorUnits<Untyped> {
    pub fn try_from_major(currency: CurrencyCode, major: Decimal) -> Result<Self, ConversionError> {
        match currency {
            CurrencyCode::USD => Ok(MinorUnits::<Usd>::try_from_major(major)?.into()),
            CurrencyCode::BTC => Ok(MinorUnits::<Btc>::try_from_major(major)?.into()),
            _ => Err(ConversionError::UnsupportedCurrency(currency)),
        }
    }

    pub fn to_typed<C: StaticCurrency>(&self) -> Result<MinorUnits<C>, ConversionError> {
        if self.currency.code() != C::CODE {
            return Err(ConversionError::CurrencyMismatch {
                expected: C::CODE,
                actual: self.currency.code(),
            });
        }
        Ok(MinorUnits {
            value: self.value,
            currency: C::INSTANCE,
        })
    }
}

impl Serialize for MinorUnits<Untyped> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Amount", 3)?;
        s.serialize_field("currency", &self.currency.code())?;
        s.serialize_field("minor_units", &self.value)?;
        s.serialize_field(
            "minor_units_per_major",
            &self.currency.minor_units_per_major(),
        )?;
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
            currency: Untyped::from_raw(raw.currency, raw.minor_units_per_major),
        })
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

#[cfg(feature = "sqlx")]
mod minor_units_sqlx {
    use sqlx::{Type, postgres::*};

    use super::*;

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

#[cfg(feature = "graphql")]
mod minor_units_graphql {
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
}

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

impl<C: StaticCurrency> Default for SignedMinorUnits<C> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<C: StaticCurrency> fmt::Display for SignedMinorUnits<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl<C: StaticCurrency> TryFrom<MinorUnits<C>> for SignedMinorUnits<C> {
    type Error = ConversionError;
    fn try_from(val: MinorUnits<C>) -> Result<Self, Self::Error> {
        Ok(Self(
            i64::try_from(val.value).map_err(|_| ConversionError::Overflow)?,
            PhantomData,
        ))
    }
}

impl<C: StaticCurrency> TryFrom<SignedMinorUnits<C>> for MinorUnits<C> {
    type Error = ConversionError;
    fn try_from(value: SignedMinorUnits<C>) -> Result<Self, Self::Error> {
        Self::try_from_major(value.to_major())
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

#[cfg(feature = "graphql")]
mod signed_minor_units_graphql {
    use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

    use super::*;

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
