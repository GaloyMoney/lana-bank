use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{LedgerAccountId, primitives::CalaTxId};

use cala_ledger::{
    BalanceId, Currency as CalaCurrency, DebitOrCredit, account_set::AccountSetId,
    balance::BalanceRange as CalaBalanceRange, velocity::VelocityLimit,
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
pub(super) struct ClosingTxEntry {
    pub(super) account_id: LedgerAccountId,
    pub(super) amount: Decimal,
    pub(super) currency: CalaCurrency,
    pub(super) direction: DebitOrCredit,
}

impl ClosingTxEntry {
    pub(super) fn new(
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

#[derive(Debug)]
pub(crate) struct ClosingTxParams {
    pub(crate) tx_id: CalaTxId,
    pub(crate) description: String,
    pub(crate) effective_balances_from: NaiveDate,
    pub(crate) effective_balances_until: NaiveDate,
    pub(crate) revenue_account_set_id: AccountSetId,
    pub(crate) cost_of_revenue_account_set_id: AccountSetId,
    pub(crate) expenses_account_set_id: AccountSetId,
    pub(crate) equity_retained_earnings_account_set_id: AccountSetId,
    pub(crate) equity_retained_losses_account_set_id: AccountSetId,
}

#[derive(Debug, Clone)]
pub(super) struct LineItemClosingTxInput {
    pub(super) net_income_contribution: Decimal,
    pub(super) entries: Vec<ClosingTxEntry>,
}

#[derive(Debug, Clone)]
pub(super) struct ProfitAndLossLineItemDetail(HashMap<BalanceId, CalaBalanceRange>);

impl ProfitAndLossLineItemDetail {
    pub(super) fn closing_tx_input(&self) -> LineItemClosingTxInput {
        let mut contribution: Decimal = Decimal::ZERO;
        let mut closing_entries = Vec::new();

        for ((_, account_id, currency), balance) in self.iter() {
            let amount = balance.close.settled();
            let balance_type = balance.close.balance_type;

            contribution += match balance_type {
                DebitOrCredit::Credit => amount,
                DebitOrCredit::Debit => -amount,
            };

            let offset_dir = if amount >= Decimal::ZERO {
                match balance_type {
                    DebitOrCredit::Debit => DebitOrCredit::Credit,
                    DebitOrCredit::Credit => DebitOrCredit::Debit,
                }
            } else {
                balance_type
            };

            closing_entries.push(ClosingTxEntry::new(
                (*account_id).into(),
                amount.abs(),
                *currency,
                offset_dir,
            ));
        }

        LineItemClosingTxInput {
            net_income_contribution: contribution,
            entries: closing_entries,
        }
    }

    pub(super) fn iter(&self) -> std::collections::hash_map::Iter<'_, BalanceId, CalaBalanceRange> {
        self.0.iter()
    }
}

impl From<HashMap<BalanceId, CalaBalanceRange>> for ProfitAndLossLineItemDetail {
    fn from(account_balances: HashMap<BalanceId, CalaBalanceRange>) -> Self {
        Self(account_balances)
    }
}

#[derive(Debug, Clone)]
pub(super) struct ClosingTxNetIncomeInput {
    pub(super) net_income: Decimal,
    pub(super) retained_earnings_parent_account_id: AccountSetId,
    pub(super) entries: Vec<ClosingTxEntry>,
    pub(super) retained_earnings_account_normal_balance_type: DebitOrCredit,
}

impl ClosingTxNetIncomeInput {
    pub(super) fn merge_closing_tx_entries(
        &self,
        account_id: LedgerAccountId,
    ) -> Vec<ClosingTxEntry> {
        let retained_earnings_entry = ClosingTxEntry {
            account_id,
            amount: self.net_income.abs(),
            currency: CalaCurrency::USD,
            direction: self.retained_earnings_account_normal_balance_type,
        };
        self.entries
            .iter()
            .cloned()
            .chain(std::iter::once(retained_earnings_entry))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ClosingProfitAndLossAccountBalances {
    pub(super) revenue: ProfitAndLossLineItemDetail,
    pub(super) cost_of_revenue: ProfitAndLossLineItemDetail,
    pub(super) expenses: ProfitAndLossLineItemDetail,
}

impl ClosingProfitAndLossAccountBalances {
    pub(super) fn to_net_income_input(
        &self,
        retained_earnings_gain_account_id: AccountSetId,
        retained_earnings_loss_account_id: AccountSetId,
    ) -> ClosingTxNetIncomeInput {
        let mut revenue_closing_input = self.revenue.closing_tx_input();
        let mut cost_of_revenue_closing_input = self.cost_of_revenue.closing_tx_input();
        let mut expenses_closing_input = self.expenses.closing_tx_input();

        let net_income = revenue_closing_input.net_income_contribution
            + cost_of_revenue_closing_input.net_income_contribution
            + expenses_closing_input.net_income_contribution;

        let mut entries = Vec::new();
        entries.append(&mut revenue_closing_input.entries);
        entries.append(&mut cost_of_revenue_closing_input.entries);
        entries.append(&mut expenses_closing_input.entries);

        let (retained_earnings_account_balance_type, retained_earnings_parent_account_id) =
            if net_income >= Decimal::ZERO {
                (DebitOrCredit::Credit, retained_earnings_gain_account_id)
            } else {
                (DebitOrCredit::Debit, retained_earnings_loss_account_id)
            };

        ClosingTxNetIncomeInput {
            net_income,
            entries,
            retained_earnings_parent_account_id,
            retained_earnings_account_normal_balance_type: retained_earnings_account_balance_type,
        }
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
