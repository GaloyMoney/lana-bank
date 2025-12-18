use std::{fmt, marker::PhantomData, str::FromStr};

use rust_decimal::Decimal;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{DomainConfigError, DomainConfigKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum SimpleType {
    Bool,
    String,
    Int,
    Decimal,
}

impl SimpleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SimpleType::Bool => "bool",
            SimpleType::String => "string",
            SimpleType::Int => "int",
            SimpleType::Decimal => "decimal",
        }
    }

    pub fn parse_json(&self, value: Value) -> Result<SimpleValue, DomainConfigError> {
        match self {
            SimpleType::Bool => match value {
                Value::Bool(v) => Ok(SimpleValue::Bool(v)),
                other => Err(DomainConfigError::InvalidSimpleValue(format!(
                    "Expected bool, got {other:?}"
                ))),
            },
            SimpleType::String => match value {
                Value::String(v) => Ok(SimpleValue::String(v)),
                other => Err(DomainConfigError::InvalidSimpleValue(format!(
                    "Expected string, got {other:?}"
                ))),
            },
            SimpleType::Int => match value {
                Value::Number(v) => v.as_i64().map(SimpleValue::Int).ok_or_else(|| {
                    DomainConfigError::InvalidSimpleValue(format!(
                        "Expected i64-compatible number, got {v}"
                    ))
                }),
                other => Err(DomainConfigError::InvalidSimpleValue(format!(
                    "Expected number, got {other:?}"
                ))),
            },
            SimpleType::Decimal => match value {
                Value::String(v) => Decimal::from_str(&v)
                    .map(SimpleValue::Decimal)
                    .map_err(|e| DomainConfigError::InvalidSimpleValue(format!("{e}"))),
                other => Err(DomainConfigError::InvalidSimpleValue(format!(
                    "Expected decimal string, got {other:?}"
                ))),
            },
        }
    }
}

impl fmt::Display for SimpleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleValue {
    Bool(bool),
    String(String),
    Int(i64),
    Decimal(Decimal),
}

impl SimpleValue {
    pub fn to_json(&self) -> Value {
        match self {
            SimpleValue::Bool(v) => Value::Bool(*v),
            SimpleValue::String(v) => Value::String(v.clone()),
            SimpleValue::Int(v) => Value::Number((*v).into()),
            SimpleValue::Decimal(v) => Value::String(v.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleConfig<T: SimpleScalar> {
    pub key: &'static str,
    _marker: PhantomData<T>,
}

impl<T: SimpleScalar> SimpleConfig<T> {
    /// # Examples
    /// ```no_run
    /// use domain_config::{DomainConfigs, SimpleConfig};
    /// use rust_decimal::Decimal;
    ///
    /// // Client-facing definitions
    /// const FEATURE_X_ENABLED: SimpleConfig<bool> = SimpleConfig::new("feature_x_enabled");
    /// const MAX_RETRIES: SimpleConfig<i64> = SimpleConfig::new("max_retries");
    /// const FEE_RATE: SimpleConfig<Decimal> = SimpleConfig::new("fee_rate");
    ///
    /// async fn configure(pool: &sqlx::PgPool) -> Result<(), domain_config::DomainConfigError> {
    ///     let configs = DomainConfigs::new(pool);
    ///
    ///     // Create strongly-typed simple configs
    ///     configs.create_simple(FEATURE_X_ENABLED, true).await?;
    ///     configs.create_simple(MAX_RETRIES, 3).await?;
    ///     configs.create_simple(FEE_RATE, Decimal::new(25, 2)).await?;
    ///
    ///     // Typed access
    ///     let enabled: bool = configs.get_simple(FEATURE_X_ENABLED).await?;
    ///     let retries: i64 = configs.get_simple(MAX_RETRIES).await?;
    ///     let fee_rate: Decimal = configs.get_simple(FEE_RATE).await?;
    ///
    ///     // Update stays type-safe
    ///     configs.update_simple(FEATURE_X_ENABLED, !enabled).await?;
    ///
    ///     // Listing returns dynamic entries
    ///     let all = configs.list_simple().await?;
    ///     assert!(all.iter().any(|c| c.key == "feature_x_enabled" && matches!(c.simple_type, domain_config::SimpleType::Bool)));
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use domain_config::{DomainConfigs, SimpleConfig};
    ///
    /// const FEATURE_X_ENABLED: SimpleConfig<bool> = SimpleConfig::new("feature_x_enabled");
    ///
    /// fn wrong_type(configs: DomainConfigs) {
    ///     let _ = configs.create_simple(FEATURE_X_ENABLED, "not a bool");
    /// }
    /// ```
    pub const fn new(key: &'static str) -> Self {
        Self {
            key,
            _marker: PhantomData,
        }
    }
}

impl<T: SimpleScalar> From<SimpleConfig<T>> for DomainConfigKey {
    fn from(spec: SimpleConfig<T>) -> Self {
        DomainConfigKey::new(spec.key)
    }
}

pub trait SimpleScalar: sealed::Sealed + Sized + fmt::Debug {
    const SIMPLE_TYPE: SimpleType;
    fn to_json(&self) -> Value;
    fn from_json(v: Value) -> Result<Self, DomainConfigError>;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for bool {}
    impl Sealed for String {}
    impl Sealed for i64 {}
    impl Sealed for rust_decimal::Decimal {}
}

impl SimpleScalar for bool {
    const SIMPLE_TYPE: SimpleType = SimpleType::Bool;

    fn to_json(&self) -> Value {
        Value::Bool(*self)
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::Bool(v) => Ok(v),
            other => Err(DomainConfigError::InvalidSimpleValue(format!(
                "Expected bool, got {other:?}"
            ))),
        }
    }
}

impl SimpleScalar for String {
    const SIMPLE_TYPE: SimpleType = SimpleType::String;

    fn to_json(&self) -> Value {
        Value::String(self.clone())
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::String(v) => Ok(v),
            other => Err(DomainConfigError::InvalidSimpleValue(format!(
                "Expected string, got {other:?}"
            ))),
        }
    }
}

impl SimpleScalar for i64 {
    const SIMPLE_TYPE: SimpleType = SimpleType::Int;

    fn to_json(&self) -> Value {
        Value::Number((*self).into())
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::Number(v) => v.as_i64().ok_or_else(|| {
                DomainConfigError::InvalidSimpleValue(format!(
                    "Expected i64-compatible number, got {v}"
                ))
            }),
            other => Err(DomainConfigError::InvalidSimpleValue(format!(
                "Expected number, got {other:?}"
            ))),
        }
    }
}

impl SimpleScalar for Decimal {
    const SIMPLE_TYPE: SimpleType = SimpleType::Decimal;

    fn to_json(&self) -> Value {
        Value::String(self.to_string())
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::String(v) => Decimal::from_str(&v)
                .map_err(|e| DomainConfigError::InvalidSimpleValue(format!("{e}"))),
            other => Err(DomainConfigError::InvalidSimpleValue(format!(
                "Expected decimal string, got {other:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimpleEntry {
    pub key: String,
    pub simple_type: SimpleType,
    pub value: SimpleValue,
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::*;

    const _BOOL_CONFIG: SimpleConfig<bool> = SimpleConfig::new("feature_x_enabled");
    const _INT_CONFIG: SimpleConfig<i64> = SimpleConfig::new("max_retries");

    #[test]
    fn bool_round_trips() {
        let json = true.to_json();
        let parsed = bool::from_json(json).unwrap();
        assert!(parsed);
    }

    #[test]
    fn decimal_is_string_encoded() {
        let val = dec!(12.34);
        let json = val.to_json();
        assert_eq!(json, Value::String("12.34".to_string()));
        let parsed = Decimal::from_json(json).unwrap();
        assert_eq!(parsed, val);
    }

    #[test]
    fn parse_json_uses_simple_type() {
        let entry = SimpleType::Int
            .parse_json(json!(123))
            .expect("Should parse int");
        assert_eq!(entry, SimpleValue::Int(123));

        let err = SimpleType::Bool.parse_json(json!("nope")).unwrap_err();
        assert!(matches!(err, DomainConfigError::InvalidSimpleValue(_)));
    }
}
