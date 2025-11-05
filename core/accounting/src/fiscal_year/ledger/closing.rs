use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct MonthlyClosing {
    pub(super) closed_as_of: chrono::NaiveDate,
    pub(super) closed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct AccountingClosingMetadata {
    pub(super) monthly: MonthlyClosing,
}

impl AccountingClosingMetadata {
    pub(super) const METADATA_KEY: &'static str = "closing";

    pub(super) fn update_metadata(
        metadata: &mut serde_json::Value,
        closed_as_of: chrono::NaiveDate,
    ) {
        let closing_metadata = Self {
            monthly: MonthlyClosing {
                closed_as_of,
                closed_at: crate::time::now(),
            },
        };

        metadata
            .as_object_mut()
            .expect("metadata should be an object")
            .insert(
                Self::METADATA_KEY.to_string(),
                serde_json::json!(closing_metadata),
            );
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use serde_json::json;

    use super::*;

    #[test]
    fn can_update_metadata_with_empty_metadata() {
        let mut metadata = json!({});
        let closed_as_of = "2024-01-31".parse::<NaiveDate>().unwrap();

        AccountingClosingMetadata::update_metadata(&mut metadata, closed_as_of);

        let closing_meta: AccountingClosingMetadata =
            serde_json::from_value(metadata["closing"].clone()).unwrap();
        assert_eq!(closing_meta.monthly.closed_as_of, closed_as_of);
    }

    #[test]
    fn can_update_metadata_with_new_closing() {
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

        let new_date = "2024-01-31".parse::<NaiveDate>().unwrap();
        AccountingClosingMetadata::update_metadata(&mut metadata, new_date);

        let closing_meta: AccountingClosingMetadata =
            serde_json::from_value(metadata["closing"].clone()).unwrap();
        assert_eq!(closing_meta.monthly.closed_as_of, new_date);
        assert!(closing_meta.monthly.closed_at != existing_time);
    }

    #[test]
    fn can_update_metadata_with_other_fields() {
        let mut metadata = json!({
            "other_field": "value",
            "another_field": 123
        });
        let closed_as_of = "2024-01-31".parse::<NaiveDate>().unwrap();

        AccountingClosingMetadata::update_metadata(&mut metadata, closed_as_of);

        assert_eq!(metadata.get("other_field").unwrap(), "value");
        assert_eq!(metadata.get("another_field").unwrap(), 123);
        assert!(metadata.get("closing").is_some());
    }
}
