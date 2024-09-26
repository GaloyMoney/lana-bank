use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Duration {
    Months(u32),
}

impl Duration {
    pub fn expiration_date(&self, start_date: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Duration::Months(months) => start_date
                .checked_add_months(chrono::Months::new(*months))
                .expect("should return a expiration date"),
        }
    }
}

#[derive(Builder, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct CreditFacilityTermValues {
    #[builder(setter(into))]
    pub(crate) duration: Duration,
}

impl CreditFacilityTermValues {
    pub fn builder() -> CreditFacilityTermValuesBuilder {
        CreditFacilityTermValuesBuilder::default()
    }
}
