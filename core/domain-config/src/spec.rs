use serde::{Serialize, de::DeserializeOwned};
use std::{marker::PhantomData, str::FromStr};

use crate::{ConfigType, DomainConfigError, DomainConfigKey, Visibility};

pub struct Simple<T>(PhantomData<T>);
pub struct Complex<T>(PhantomData<T>);

// Prevent downstream crates from defining new config kinds
mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for super::Simple<T> {}
    impl<T> Sealed for super::Complex<T> {}
}

pub trait ValueKind: sealed::Sealed {
    type Value: Clone;
    const TYPE: ConfigType;

    fn encode(value: &Self::Value) -> Result<serde_json::Value, DomainConfigError>;
    fn decode(value: serde_json::Value) -> Result<Self::Value, DomainConfigError>;
}

impl ValueKind for Simple<bool> {
    type Value = bool;
    const TYPE: ConfigType = ConfigType::Bool;

    fn encode(value: &Self::Value) -> Result<serde_json::Value, DomainConfigError> {
        Ok(serde_json::Value::Bool(*value))
    }

    fn decode(value: serde_json::Value) -> Result<Self::Value, DomainConfigError> {
        match value {
            serde_json::Value::Bool(value) => Ok(value),
            other => Err(DomainConfigError::InvalidType(format!(
                "Expected bool, got {}",
                json_value_type(&other)
            ))),
        }
    }
}

impl ValueKind for Simple<String> {
    type Value = String;
    const TYPE: ConfigType = ConfigType::String;

    fn encode(value: &Self::Value) -> Result<serde_json::Value, DomainConfigError> {
        Ok(serde_json::Value::String(value.clone()))
    }

    fn decode(value: serde_json::Value) -> Result<Self::Value, DomainConfigError> {
        match value {
            serde_json::Value::String(value) => Ok(value),
            other => Err(DomainConfigError::InvalidType(format!(
                "Expected string, got {}",
                json_value_type(&other)
            ))),
        }
    }
}

impl ValueKind for Simple<i64> {
    type Value = i64;
    const TYPE: ConfigType = ConfigType::Int;

    fn encode(value: &Self::Value) -> Result<serde_json::Value, DomainConfigError> {
        Ok(serde_json::Value::Number((*value).into()))
    }

    fn decode(value: serde_json::Value) -> Result<Self::Value, DomainConfigError> {
        value.as_i64().ok_or_else(|| {
            DomainConfigError::InvalidType(format!("Expected i64, got {}", json_value_type(&value)))
        })
    }
}

impl ValueKind for Simple<u64> {
    type Value = u64;
    const TYPE: ConfigType = ConfigType::Uint;

    fn encode(value: &Self::Value) -> Result<serde_json::Value, DomainConfigError> {
        Ok(serde_json::Value::Number((*value).into()))
    }

    fn decode(value: serde_json::Value) -> Result<Self::Value, DomainConfigError> {
        value.as_u64().ok_or_else(|| {
            DomainConfigError::InvalidType(format!("Expected u64, got {}", json_value_type(&value)))
        })
    }
}

impl ValueKind for Simple<rust_decimal::Decimal> {
    type Value = rust_decimal::Decimal;
    const TYPE: ConfigType = ConfigType::Decimal;

    fn encode(value: &Self::Value) -> Result<serde_json::Value, DomainConfigError> {
        Ok(serde_json::Value::String(value.to_string()))
    }

    fn decode(value: serde_json::Value) -> Result<Self::Value, DomainConfigError> {
        match &value {
            serde_json::Value::String(value) => {
                rust_decimal::Decimal::from_str(value).map_err(|_| {
                    DomainConfigError::InvalidType(
                        "Expected decimal string, got invalid string".to_string(),
                    )
                })
            }
            other => Err(DomainConfigError::InvalidType(format!(
                "Expected decimal string, got {}",
                json_value_type(other)
            ))),
        }
    }
}

impl<T> ValueKind for Complex<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    type Value = T;
    const TYPE: ConfigType = ConfigType::Complex;

    fn encode(value: &Self::Value) -> Result<serde_json::Value, DomainConfigError> {
        Ok(serde_json::to_value(value)?)
    }

    fn decode(value: serde_json::Value) -> Result<Self::Value, DomainConfigError> {
        Ok(serde_json::from_value(value)?)
    }
}

pub trait ConfigSpec {
    const KEY: DomainConfigKey;
    const VISIBILITY: Visibility;
    type Kind: ValueKind;

    fn default_value() -> Option<<Self::Kind as ValueKind>::Value> {
        None
    }

    fn validate(_: &<Self::Kind as ValueKind>::Value) -> Result<(), DomainConfigError> {
        Ok(())
    }

    fn validate_json(value: &serde_json::Value) -> Result<(), DomainConfigError> {
        if value.is_null() {
            return Err(DomainConfigError::InvalidState(format!(
                "Expected non-null value for {}",
                Self::KEY.as_str()
            )));
        }

        let decoded = <Self::Kind as ValueKind>::decode(value.clone())?;
        Self::validate(&decoded)?;

        Ok(())
    }
}

fn json_value_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
