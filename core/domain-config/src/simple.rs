use rust_decimal::Decimal;
use serde_json::Value;
use std::{fmt, str::FromStr};

use crate::{
    DomainConfigError,
    primitives::{ConfigType, DomainConfigKey},
};

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
///     assert!(all.iter().any(|c| c.key == domain_config::DomainConfigKey::new("feature_x_enabled") && matches!(c.config_type, domain_config::ConfigType::Bool)));
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
    const CONFIG_TYPE: ConfigType;
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
    const CONFIG_TYPE: ConfigType = ConfigType::Bool;

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
    const CONFIG_TYPE: ConfigType = ConfigType::String;

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
    const CONFIG_TYPE: ConfigType = ConfigType::Int;

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
    const CONFIG_TYPE: ConfigType = ConfigType::Decimal;

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
    pub config_type: ConfigType,
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
