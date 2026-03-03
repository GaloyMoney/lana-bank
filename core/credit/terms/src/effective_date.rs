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

impl From<EffectiveDate> for chrono::NaiveDate {
    fn from(date: EffectiveDate) -> Self {
        date.0
    }
}

impl EffectiveDate {
    /// Returns the last representable instant of this calendar day in UTC.
    ///
    /// Uses nanosecond precision (23:59:59.999_999_999) to minimize the gap
    /// before midnight, avoiding the previous 1-second gap at 23:59:59.
    pub fn end_of_day(&self) -> DateTime<Utc> {
        Utc.from_utc_datetime(
            &self
                .0
                .and_hms_nano_opt(23, 59, 59, 999_999_999)
                .expect("23:59:59.999999999 was invalid"),
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
