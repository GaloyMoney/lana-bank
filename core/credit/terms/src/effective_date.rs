use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EffectiveDate(chrono::NaiveDate);

impl From<chrono::NaiveDate> for EffectiveDate {
    fn from(date: chrono::NaiveDate) -> Self {
        Self(date)
    }
}

impl From<DateTime<Utc>> for EffectiveDate {
    fn from(date: DateTime<Utc>) -> Self {
        Self(date.date_naive())
    }
}

impl EffectiveDate {
    pub fn end_of_day(&self) -> DateTime<Utc> {
        Utc.from_utc_datetime(
            &self
                .0
                .and_hms_opt(23, 59, 59)
                .expect("23:59:59 was invalid"),
        )
    }

    pub fn start_of_day(&self) -> DateTime<Utc> {
        Utc.from_utc_datetime(
            &self
                .0
                .and_hms_opt(00, 00, 00)
                .expect("00:00:00 was invalid"),
        )
    }

    pub fn checked_add_days(&self, days: chrono::Days) -> Option<Self> {
        self.0.checked_add_days(days).map(Self)
    }
}
