use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{fmt, str::FromStr};

// ---------------------------------------------------------------------------
// CurrencyCode — runtime currency identifier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurrencyCode(&'static str);

#[cfg(feature = "json-schema")]
impl schemars::JsonSchema for CurrencyCode {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        String::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}

impl CurrencyCode {
    pub const USD: Self = Self("USD");
    pub const BTC: Self = Self("BTC");

    const ALL: &[Self] = &[Self::USD, Self::BTC];

    pub fn iso(&self) -> &'static str {
        self.0
    }

    pub fn minor_units_per_major(self) -> u64 {
        match self {
            Self::USD => 100,
            Self::BTC => 100_000_000,
            _ => panic!("unknown currency: {}", self.0),
        }
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<CurrencyCode> for &'static str {
    fn from(code: CurrencyCode) -> Self {
        code.0
    }
}

impl FromStr for CurrencyCode {
    type Err = ParseCurrencyCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .find(|c| c.0 == s)
            .copied()
            .ok_or_else(|| ParseCurrencyCodeError::UnknownCurrencyCode(s.to_string()))
    }
}

impl Serialize for CurrencyCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CurrencyCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for CurrencyCode {
    type Error = ParseCurrencyCodeError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum ParseCurrencyCodeError {
    #[error("ParseCurrencyCodeError - UnknownCurrencyCode: {0}")]
    UnknownCurrencyCode(String),
}
