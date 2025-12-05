use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::LedgerAccountId;

use cala_ledger::{
    BalanceId, Currency as CalaCurrency, DebitOrCredit, balance::BalanceRange as CalaBalanceRange,
    velocity::VelocityLimit,
};

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

    fn period_cel_conditions(period: &str) -> String {
        format!(
            r#"
            !has({path}) ||
            !has({path}.{key}) ||
            !has({path}.{key}.{period}) ||
            !has({path}.{key}.{period}.{closing_date_key}) ||
            date({path}.{key}.{period}.{closing_date_key}) >= context.vars.transaction.effective
        "#,
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
}

#[derive(Debug, Clone)]
pub struct ClosingAccountEntry {
    pub account_id: LedgerAccountId,
    pub amount: Decimal,
    pub currency: CalaCurrency,
    pub direction: DebitOrCredit,
}

impl ClosingAccountEntry {
    pub fn new(
        account_id: LedgerAccountId,
        amount: Decimal,
        currency: CalaCurrency,
        direction: DebitOrCredit,
    ) -> Self {
        Self {
            account_id,
            amount,
            currency,
            direction,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClosingAccountBalances {
    pub revenue: HashMap<BalanceId, CalaBalanceRange>,
    pub cost_of_revenue: HashMap<BalanceId, CalaBalanceRange>,
    pub expenses: HashMap<BalanceId, CalaBalanceRange>,
}

impl ClosingAccountBalances {
    pub fn to_closing_entries(&self) -> (Decimal, Vec<ClosingAccountEntry>) {
        let (revenue_balance, mut revenue) = Self::create_closing_account_entries(&self.revenue);
        let (cost_of_revenue_balance, mut cost_of_revenue) =
            Self::create_closing_account_entries(&self.cost_of_revenue);
        let (expenses_balance, mut expenses) = Self::create_closing_account_entries(&self.expenses);

        let net_income = revenue_balance - cost_of_revenue_balance - expenses_balance;

        let mut entries = Vec::new();
        entries.append(&mut revenue);
        entries.append(&mut expenses);
        entries.append(&mut cost_of_revenue);

        (net_income, entries)
    }

    fn create_closing_account_entries(
        balances: &HashMap<BalanceId, CalaBalanceRange>,
    ) -> (Decimal, Vec<ClosingAccountEntry>) {
        let mut net_balance: Decimal = Decimal::ZERO;
        let mut closing_entries = Vec::new();

        for ((_, account_id, currency), balance) in balances {
            let amount = balance.close.settled().abs();
            net_balance += amount;
            let direction = if balance.close.balance_type == DebitOrCredit::Debit {
                DebitOrCredit::Credit
            } else {
                DebitOrCredit::Debit
            };

            closing_entries.push(ClosingAccountEntry::new(
                (*account_id).into(),
                amount,
                *currency,
                direction,
            ));
        }

        (net_balance, closing_entries)
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
