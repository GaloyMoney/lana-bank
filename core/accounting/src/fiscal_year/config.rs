use serde::{Deserialize, Serialize};
use thiserror::Error;

use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::AccountCode;
use domain_config::{DomainConfigKey, DomainConfigValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(try_from = "u8", into = "u8")]
pub struct YearEndMonth(u8);

#[derive(Error, Debug)]
pub enum YearEndMonthParseError {
    #[error("value must represent calendar month (1-12), got {0}")]
    OutOfRange(u8),
}

impl ErrorSeverity for YearEndMonthParseError {
    fn severity(&self) -> Level {
        match self {
            Self::OutOfRange(_) => Level::ERROR,
        }
    }
}

impl TryFrom<u8> for YearEndMonth {
    type Error = YearEndMonthParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=12).contains(&value) {
            return Err(YearEndMonthParseError::OutOfRange(value));
        }
        Ok(YearEndMonth(value))
    }
}

impl From<YearEndMonth> for u8 {
    fn from(value: YearEndMonth) -> Self {
        value.0
    }
}

impl YearEndMonth {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalYearConfig {
    pub year_end_month: YearEndMonth,
    pub revenue_code: AccountCode,
    pub cost_of_revenue_code: AccountCode,
    pub expenses_code: AccountCode,
    pub equity_retained_earnings_code: AccountCode,
    pub equity_retained_losses_code: AccountCode,
    // TODO: some type of frequency of closing type indication -
    // "monthly"/"quarterly" or "soft"/"hard" - especially if considering many jurisdictions
    // now.
    // closing_type_or_frequency: String,
}

impl DomainConfigValue for FiscalYearConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("fiscal-year-config");
}
