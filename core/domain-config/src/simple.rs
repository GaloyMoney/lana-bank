use rust_decimal::Decimal;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, str::FromStr};

use crate::{DomainConfigError, primitives::DomainConfigKey};

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

    pub fn format_json_value(&self, value: &Value) -> Result<String, DomainConfigError> {
        match self {
            SimpleType::Bool => match value {
                Value::Bool(v) => Ok(v.to_string()),
                other => Err(DomainConfigError::InvalidType(format!(
                    "Expected bool, got {other:?}"
                ))),
            },
            SimpleType::String => match value {
                Value::String(v) => Ok(v.clone()),
                other => Err(DomainConfigError::InvalidType(format!(
                    "Expected string, got {other:?}"
                ))),
            },
            SimpleType::Int => match value {
                Value::Number(v) => v.as_i64().map(|v| v.to_string()).ok_or_else(|| {
                    DomainConfigError::InvalidType(format!(
                        "Expected i64-compatible number, got {v}"
                    ))
                }),
                other => Err(DomainConfigError::InvalidType(format!(
                    "Expected number, got {other:?}"
                ))),
            },
            SimpleType::Decimal => match value {
                Value::String(v) => Ok(v.clone()),
                other => Err(DomainConfigError::InvalidType(format!(
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

/// Marker trait describing a simple config.
///
/// # Examples
/// ```no_run
/// use domain_config::{DomainConfigs, DomainConfigKey, SimpleConfig};
/// use rust_decimal::Decimal;
///
/// pub struct FeatureXEnabled;
/// impl SimpleConfig for FeatureXEnabled {
///     type Scalar = bool;
///     const KEY: DomainConfigKey = DomainConfigKey::new("feature_x_enabled");
/// }
///
/// pub struct MaxRetries;
/// impl SimpleConfig for MaxRetries {
///     type Scalar = i64;
///     const KEY: DomainConfigKey = DomainConfigKey::new("max_retries");
/// }
///
/// pub struct FeeRate;
/// impl SimpleConfig for FeeRate {
///     type Scalar = Decimal;
///     const KEY: DomainConfigKey = DomainConfigKey::new("fee_rate");
/// }
///
/// async fn configure(pool: &sqlx::PgPool) -> Result<(), domain_config::DomainConfigError> {
///     let configs = DomainConfigs::new(pool);
///
///     // Create strongly-typed simple configs
///     configs.create_simple::<FeatureXEnabled>(true).await?;
///     configs.create_simple::<MaxRetries>(3).await?;
///     configs.create_simple::<FeeRate>(Decimal::new(25, 2)).await?;
///
///     // Typed access
///     let enabled: bool = configs.get_simple::<FeatureXEnabled>().await?;
///     let retries: i64 = configs.get_simple::<MaxRetries>().await?;
///     let fee_rate: Decimal = configs.get_simple::<FeeRate>().await?;
///
///     // Update stays type-safe
///     configs.update_simple::<FeatureXEnabled>(!enabled).await?;
///
///     // Listing returns dynamic entries
///     let all = configs.list_simple().await?;
///     assert!(all.iter().any(|c| c.key == domain_config::DomainConfigKey::new("feature_x_enabled") && matches!(c.simple_type, domain_config::SimpleType::Bool)));
///
///     Ok(())
/// }
/// ```
///
/// ```compile_fail
/// use domain_config::{DomainConfigs, DomainConfigKey, SimpleConfig};
///
/// pub struct FeatureXEnabled;
/// impl SimpleConfig for FeatureXEnabled {
///     type Scalar = bool;
///     const KEY: DomainConfigKey = DomainConfigKey::new("feature_x_enabled");
/// }
///
/// fn wrong_type(configs: DomainConfigs) {
///     let _ = configs.create_simple::<FeatureXEnabled>("not a bool");
/// }
/// ```
pub trait SimpleConfig {
    type Scalar: SimpleScalar;
    const KEY: DomainConfigKey;
}

pub trait SimpleScalar: sealed::Sealed + Sized + fmt::Debug + Clone {
    const SIMPLE_TYPE: SimpleType;
    fn to_json(v: &Self) -> Value;
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

    fn to_json(v: &Self) -> Value {
        Value::Bool(*v)
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::Bool(v) => Ok(v),
            other => Err(DomainConfigError::InvalidType(format!(
                "Expected bool, got {other:?}"
            ))),
        }
    }
}

impl SimpleScalar for String {
    const SIMPLE_TYPE: SimpleType = SimpleType::String;

    fn to_json(v: &Self) -> Value {
        Value::String(v.clone())
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::String(v) => Ok(v),
            other => Err(DomainConfigError::InvalidType(format!(
                "Expected string, got {other:?}"
            ))),
        }
    }
}

impl SimpleScalar for i64 {
    const SIMPLE_TYPE: SimpleType = SimpleType::Int;

    fn to_json(v: &Self) -> Value {
        Value::Number((*v).into())
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::Number(v) => v.as_i64().ok_or_else(|| {
                DomainConfigError::InvalidType(format!("Expected i64-compatible number, got {v}"))
            }),
            other => Err(DomainConfigError::InvalidType(format!(
                "Expected number, got {other:?}"
            ))),
        }
    }
}

impl SimpleScalar for Decimal {
    const SIMPLE_TYPE: SimpleType = SimpleType::Decimal;

    fn to_json(v: &Self) -> Value {
        Value::String(v.to_string())
    }

    fn from_json(v: Value) -> Result<Self, DomainConfigError> {
        match v {
            Value::String(v) => Ok(Decimal::from_str(&v)?),
            other => Err(DomainConfigError::InvalidType(format!(
                "Expected decimal string, got {other:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimpleEntry {
    pub key: DomainConfigKey,
    pub simple_type: SimpleType,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn bool_round_trips() {
        let json = bool::to_json(&true);
        let parsed = bool::from_json(json).unwrap();
        assert!(parsed);
    }

    #[test]
    fn decimal_is_string_encoded() {
        let val = dec!(12.34);
        let json = Decimal::to_json(&val);
        assert_eq!(json, Value::String("12.34".to_string()));
        let parsed = Decimal::from_json(json).unwrap();
        assert_eq!(parsed, val);
    }
}
