use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct AccountingClosingMetadata;

impl AccountingClosingMetadata {
    const METADATA_KEY: &'static str = "closing";
    const CLOSING_DATE_KEY: &'static str = "closed_as_of";

    const MONTHLY: &'static str = "monthly";

    fn update_with_period_closing(
        period: &str,
        metadata: &mut serde_json::Value,
        closed_as_of: chrono::NaiveDate,
    ) {
        let closing_metadata = serde_json::json!({
            period: {
                Self::CLOSING_DATE_KEY: closed_as_of,
                "closed_at": crate::time::now(),
            }
        });

        metadata
            .as_object_mut()
            .expect("metadata should be an object")
            .insert(Self::METADATA_KEY.to_string(), closing_metadata);
    }

    pub(super) fn update_with_monthly_closing(
        metadata: &mut serde_json::Value,
        closed_as_of: chrono::NaiveDate,
    ) {
        Self::update_with_period_closing(Self::MONTHLY, metadata, closed_as_of)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod update_with_monthly_closing {
        use chrono::{DateTime, NaiveDate, Utc};
        use serde_json::json;

        use super::*;

        #[test]
        fn can_update_with_monthly_closing_with_empty_metadata() {
            let mut metadata = json!({});
            let closed_as_of = "2024-01-31".parse::<NaiveDate>().unwrap();

            AccountingClosingMetadata::update_with_monthly_closing(&mut metadata, closed_as_of);

            assert_eq!(
                metadata["closing"]["monthly"]["closed_as_of"],
                serde_json::Value::String(closed_as_of.to_string())
            );
        }

        #[test]
        fn can_update_with_monthly_closing_with_new_closing() {
            let existing_date = "2023-12-31";
            let existing_time = "2023-12-31T18:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let mut metadata = json!({
                "closing": {
                    "monthly": {
                        "closed_as_of": existing_date,
                        "closed_at": existing_time
                    }
                }
            });

            assert_eq!(
                metadata["closing"]["monthly"]["closed_as_of"],
                serde_json::Value::String(existing_date.to_string())
            );

            let new_date = "2024-01-31".parse::<NaiveDate>().unwrap();
            AccountingClosingMetadata::update_with_monthly_closing(&mut metadata, new_date);

            assert_eq!(
                metadata["closing"]["monthly"]["closed_as_of"],
                serde_json::Value::String(new_date.to_string())
            );
        }

        #[test]
        fn can_update_with_monthly_closing_with_other_fields() {
            let mut metadata = json!({
                "other_field": "value",
                "another_field": 123
            });
            let closed_as_of = "2024-01-31".parse::<NaiveDate>().unwrap();

            AccountingClosingMetadata::update_with_monthly_closing(&mut metadata, closed_as_of);

            assert_eq!(metadata.get("other_field").unwrap(), "value");
            assert_eq!(metadata.get("another_field").unwrap(), 123);
            assert!(metadata.get("closing").is_some());
        }
    }
}
