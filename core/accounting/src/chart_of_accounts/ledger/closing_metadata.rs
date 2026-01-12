use serde::{Deserialize, Serialize};

use cala_ledger::velocity::VelocityLimit;
use es_entity::clock::Clock;

pub(super) struct AccountClosingLimits {
    pub(super) debit_settled: VelocityLimit,
    pub(super) debit_pending: VelocityLimit,
    pub(super) credit_settled: VelocityLimit,
    pub(super) credit_pending: VelocityLimit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct AccountingClosingMetadata;

impl AccountingClosingMetadata {
    const METADATA_PATH: &'static str = "context.vars.account.metadata";
    const METADATA_KEY: &'static str = "closing";
    const CLOSING_DATE_KEY: &'static str = "closed_as_of";
    const TX_METADATA_KEY: &'static str = "is_closing_tx";

    const MONTHLY: &'static str = "monthly";

    fn update_with_period_closing(
        period: &str,
        metadata: &mut serde_json::Value,
        closed_as_of: chrono::NaiveDate,
    ) {
        let closing_metadata = serde_json::json!({
            period: {
                Self::CLOSING_DATE_KEY: closed_as_of,
                "closed_at": Clock::now(),
            }
        });

        metadata
            .as_object_mut()
            .expect("metadata should be an object")
            .insert(Self::METADATA_KEY.to_string(), closing_metadata);
    }

    fn period_cel_conditions(period: &str) -> String {
        format!(
            r#"
            (
             !has(context.vars.transaction.metadata) ||
             !has(context.vars.transaction.metadata.{is_closing_key}) ||
             !context.vars.transaction.metadata.{is_closing_key}
            ) &&
            (
             !has({path}) ||
             !has({path}.{key}) ||
             !has({path}.{key}.{period}) ||
             !has({path}.{key}.{period}.{closing_date_key}) ||
             date({path}.{key}.{period}.{closing_date_key}) >= context.vars.transaction.effective
            )
        "#,
            is_closing_key = Self::TX_METADATA_KEY,
            path = Self::METADATA_PATH,
            key = Self::METADATA_KEY,
            closing_date_key = Self::CLOSING_DATE_KEY,
        )
    }

    pub(super) fn update_with_monthly_closing(
        metadata: &mut serde_json::Value,
        closed_as_of: chrono::NaiveDate,
    ) {
        Self::update_with_period_closing(Self::MONTHLY, metadata, closed_as_of)
    }

    pub(super) fn monthly_cel_conditions() -> String {
        Self::period_cel_conditions(Self::MONTHLY)
    }

    pub(super) fn closing_tx_metadata_json() -> serde_json::Value {
        serde_json::json!({
            Self::TX_METADATA_KEY: true
        })
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

        fn ctx(
            account_json: serde_json::Value,
            tx_effective_date: NaiveDate,
            is_closing_tx: bool,
        ) -> CelContext {
            let mut transaction = CelMap::new();
            transaction.insert("effective", CelValue::Date(tx_effective_date));
            if is_closing_tx {
                transaction.insert("metadata", json!({"is_closing_tx": true}));
            }
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
            let ctx = ctx(
                account,
                AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap(),
                false,
            );

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(!block_txn);
        }

        #[test]
        fn blocks_tx_for_no_metadata() {
            let account = json!({});
            let ctx = ctx(
                account,
                AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap(),
                false,
            );

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
            let ctx = ctx(
                account,
                AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap(),
                false,
            );

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
            let ctx = ctx(
                account,
                AFTER_CLOSING_DATE.parse::<NaiveDate>().unwrap(),
                false,
            );

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
            let ctx = ctx(account, CLOSING_DATE.parse::<NaiveDate>().unwrap(), false);

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
            let ctx = ctx(
                account,
                BEFORE_CLOSING_DATE.parse::<NaiveDate>().unwrap(),
                false,
            );

            let block_txn = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(block_txn);
        }

        #[test]
        fn allows_closing_tx_after_last_month_closing() {
            let account = json!({
                "metadata": {
                    "closing": {
                        "monthly": {
                            "closed_as_of": CLOSING_DATE
                        }
                    }
                }
            });
            let ctx = ctx(account, CLOSING_DATE.parse::<NaiveDate>().unwrap(), true);
            let apply_limits = expr().try_evaluate::<bool>(&ctx).unwrap();
            assert!(
                !apply_limits,
                "Closing transaction should bypass velocity controls for closing period"
            );
        }
    }

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
