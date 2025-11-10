use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use cala_ledger::velocity::VelocityLimit;

pub(super) struct AccountClosingLimits {
    pub(super) debit_settled: VelocityLimit,
    pub(super) debit_pending: VelocityLimit,
    pub(super) credit_settled: VelocityLimit,
    pub(super) credit_pending: VelocityLimit,
}

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
    pub(super) const METADATA_PATH: &'static str = "context.vars.account.metadata";
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

    pub(super) fn monthly_cel_conditions() -> String {
        format!(
            r#"
            !has({path}) ||
            !has({path}.{key}) ||
            !has({path}.{key}.monthly) ||
            !has({path}.{key}.monthly.closed_as_of) ||
            date({path}.{key}.monthly.closed_as_of) >= context.vars.transaction.effective
        "#,
            path = Self::METADATA_PATH,
            key = Self::METADATA_KEY,
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod monthly_cel {

        use cala_cel_interpreter::{CelContext, CelExpression, CelMap, CelValue};
        use chrono::NaiveDate;
        use serde_json::json;

        use super::*;

        const CLOSING_DATE: &str = "2024-12-31";
        const BEFORE_CLOSING_DATE: &str = "2024-12-01";
        const AFTER_CLOSING_DATE: &str = "2025-01-01";

        fn expr() -> CelExpression {
            let cel_conditions = AccountingClosingMetadata::monthly_cel_conditions();
            CelExpression::try_from(cel_conditions.as_str()).unwrap()
        }

        fn ctx(account_json: serde_json::Value, tx_effective_date: NaiveDate) -> CelContext {
            let mut transaction = CelMap::new();
            transaction.insert("effective", CelValue::Date(tx_effective_date));

            let mut vars = CelMap::new();
            vars.insert("account", account_json);
            vars.insert("transaction", transaction);

            let mut context = CelMap::new();
            context.insert("vars", vars);

            let mut ctx = CelContext::new();
            ctx.add_variable("context", context);

            ctx
        }

        #[test]
        fn monthly_cel_conditions_can_be_parsed() {
            let cel_conditions = AccountingClosingMetadata::monthly_cel_conditions();
            let res = CelExpression::try_from(cel_conditions.as_str());
            assert!(res.is_ok())
        }

        #[test]
        fn allows_tx_after_monthly_closing_date() {
            let account = json!({
                "metadata": {
                    "closing": {
                        "monthly": {
                            "closed_as_of": CLOSING_DATE
                        }
                    }
                }
            });
            let ctx = ctx(account, AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap());

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(!block_txn);
        }

        #[test]
        fn blocks_tx_for_no_metadata() {
            let account = json!({});
            let ctx = ctx(account, AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap());

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(block_txn);
        }

        #[test]
        fn blocks_tx_for_no_closing_metadata() {
            let account = json!({
                "metadata": {
                    "other_field": "value"
                }
            });
            let ctx = ctx(account, AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap());

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(block_txn);
        }

        #[test]
        fn blocks_tx_for_no_monthly_closing_metadata() {
            let account = json!({
                "metadata": {
                    "closing": {
                        "other_field": "value"
                    }
                }
            });
            let ctx = ctx(account, AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap());

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(block_txn);
        }

        #[test]
        fn blocks_tx_on_monthly_closing_date() {
            let account = json!({
                "metadata": {
                    "closing": {
                        "monthly": {
                            "closed_as_of": CLOSING_DATE
                        }
                    }
                }
            });
            let ctx = ctx(account, CLOSING_DATE.parse::<NaiveDate>().unwrap());

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(block_txn);
        }

        #[test]
        fn blocks_tx_before_monthly_closing_date() {
            let account = json!({
                "metadata": {
                    "closing": {
                        "monthly": {
                            "closed_as_of": CLOSING_DATE
                        }
                    }
                }
            });
            let ctx = ctx(account, BEFORE_CLOSING_DATE.parse::<NaiveDate>().unwrap());

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(block_txn);
        }
    }

    mod update_metadata {
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
}
