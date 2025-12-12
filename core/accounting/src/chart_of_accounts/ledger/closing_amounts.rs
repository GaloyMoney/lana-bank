use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::primitives::{CalaAccount, CalaCurrency, CalaTxId};

use cala_ledger::{
    BalanceId, DebitOrCredit, account_set::AccountSetId, balance::BalanceRange as CalaBalanceRange,
};

use super::template::EntryParams;

struct AdjustedDebitOrCredit(DebitOrCredit);

impl AdjustedDebitOrCredit {
    fn new(balance: &CalaBalanceRange) -> Self {
        let amount = balance.close.settled();
        let balance_type = balance.close.balance_type;

        Self(if amount >= Decimal::ZERO {
            match balance_type {
                DebitOrCredit::Debit => DebitOrCredit::Credit,
                DebitOrCredit::Credit => DebitOrCredit::Debit,
            }
        } else {
            balance_type
        })
    }
}

impl From<AdjustedDebitOrCredit> for DebitOrCredit {
    fn from(offset_dir: AdjustedDebitOrCredit) -> Self {
        offset_dir.0
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
pub(super) struct ProfitAndLossLineItemDetail(HashMap<BalanceId, CalaBalanceRange>);

impl ProfitAndLossLineItemDetail {
    pub(super) fn contribution(&self) -> Decimal {
        self.0
            .values()
            .map(|balance| {
                let amount = balance.close.settled();
                match balance.close.balance_type {
                    DebitOrCredit::Credit => amount,
                    DebitOrCredit::Debit => -amount,
                }
            })
            .sum()
    }

    pub(super) fn entries_params(&self) -> Vec<EntryParams> {
        self.iter()
            .map(|((_, account_id, currency), balance)| {
                EntryParams::builder()
                    .account_id(*account_id)
                    .amount(balance.close.settled().abs())
                    .currency(*currency)
                    .direction(AdjustedDebitOrCredit::new(balance).into())
                    .build()
                    .expect("Failed to build EntryParams")
            })
            .collect()
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
pub(super) struct ClosingProfitAndLossAccountBalances {
    pub(super) revenue: ProfitAndLossLineItemDetail,
    pub(super) cost_of_revenue: ProfitAndLossLineItemDetail,
    pub(super) expenses: ProfitAndLossLineItemDetail,
}

impl ClosingProfitAndLossAccountBalances {
    pub(super) fn net_income(&self) -> Decimal {
        self.revenue.contribution()
            + self.cost_of_revenue.contribution()
            + self.expenses.contribution()
    }

    pub(super) fn entries_params(
        &self,
        retained_earnings_account: CalaAccount,
    ) -> Vec<EntryParams> {
        let retained_earning_entry_params =
            vec![self.retained_earnings_entry_params(retained_earnings_account)];

        self.revenue
            .entries_params()
            .into_iter()
            .chain(self.cost_of_revenue.entries_params())
            .chain(self.expenses.entries_params())
            .chain(retained_earning_entry_params)
            .collect()
    }

    fn retained_earnings_entry_params(&self, account: CalaAccount) -> EntryParams {
        EntryParams::builder()
            .account_id(account.id())
            .amount(self.net_income().abs())
            .currency(CalaCurrency::USD)
            .direction(account.values().normal_balance_type)
            .build()
            .expect("Failed to build EntryParams")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_balance_range(
        settled_amount: Decimal,
        balance_type: DebitOrCredit,
    ) -> CalaBalanceRange {
        use cala_ledger::balance::{BalanceAmount, BalanceSnapshot};
        use chrono::Utc;

        let time = Utc::now();
        let entry_id = cala_ledger::EntryId::new();
        let journal_id = cala_ledger::JournalId::new();
        let account_id = cala_ledger::AccountId::new();
        let currency: cala_ledger::Currency = "USD".parse().unwrap();

        let (dr_balance, cr_balance) = if settled_amount >= Decimal::ZERO {
            match balance_type {
                DebitOrCredit::Debit => (settled_amount, Decimal::ZERO),
                DebitOrCredit::Credit => (Decimal::ZERO, settled_amount),
            }
        } else {
            // Negative settled amount: put absolute value on opposite side
            match balance_type {
                DebitOrCredit::Debit => (Decimal::ZERO, settled_amount.abs()),
                DebitOrCredit::Credit => (settled_amount.abs(), Decimal::ZERO),
            }
        };

        let settled = BalanceAmount {
            dr_balance,
            cr_balance,
            entry_id,
            modified_at: time,
        };

        let zero_amount = BalanceAmount {
            dr_balance: Decimal::ZERO,
            cr_balance: Decimal::ZERO,
            entry_id,
            modified_at: time,
        };

        let snapshot = BalanceSnapshot {
            journal_id,
            account_id,
            entry_id,
            currency,
            settled,
            pending: zero_amount.clone(),
            encumbrance: zero_amount,
            version: 1,
            modified_at: time,
            created_at: time,
        };

        let account_balance = cala_ledger::balance::AccountBalance {
            balance_type,
            details: snapshot,
        };
        CalaBalanceRange::new(None, account_balance, 1)
    }

    mod profit_and_loss_line_item_detail {
        use super::*;
        use rust_decimal_macros::dec;

        fn create_detail_with_balance(
            amount: Decimal,
            direction: DebitOrCredit,
        ) -> ProfitAndLossLineItemDetail {
            let balance_range = create_balance_range(amount, direction);
            let journal_id = cala_ledger::JournalId::new();
            let account_id = cala_ledger::AccountId::new();
            let currency: cala_ledger::Currency = "USD".parse().unwrap();
            let mut map = HashMap::new();
            map.insert((journal_id, account_id, currency), balance_range);
            ProfitAndLossLineItemDetail::from(map)
        }

        #[test]
        fn contribution_for_credit_balance_is_positive() {
            let detail = create_detail_with_balance(dec!(1000), DebitOrCredit::Credit);
            assert_eq!(detail.contribution(), dec!(1000));
        }

        #[test]
        fn contribution_for_debit_balance_is_negative() {
            let detail = create_detail_with_balance(dec!(500), DebitOrCredit::Debit);
            assert_eq!(detail.contribution(), dec!(-500));
        }

        #[test]
        fn contribution_for_empty_detail_is_zero() {
            let detail = ProfitAndLossLineItemDetail::from(HashMap::new());
            assert_eq!(detail.contribution(), dec!(0));
        }

        #[test]
        fn contribution_sums_multiple_balances() {
            let mut map = HashMap::new();

            // Add two credit balances
            let balance1 = create_balance_range(dec!(1000), DebitOrCredit::Credit);
            let journal_id1 = cala_ledger::JournalId::new();
            let account_id1 = cala_ledger::AccountId::new();
            let currency: cala_ledger::Currency = "USD".parse().unwrap();
            map.insert((journal_id1, account_id1, currency), balance1);

            let balance2 = create_balance_range(dec!(500), DebitOrCredit::Credit);
            let journal_id2 = cala_ledger::JournalId::new();
            let account_id2 = cala_ledger::AccountId::new();
            map.insert((journal_id2, account_id2, currency), balance2);

            let detail = ProfitAndLossLineItemDetail::from(map);
            assert_eq!(detail.contribution(), dec!(1500));
        }

        #[test]
        fn entry_params_generates_closing_entry_for_credit_balance() {
            let detail = create_detail_with_balance(dec!(1000), DebitOrCredit::Credit);
            let entries_params = detail.entries_params();

            assert_eq!(entries_params.len(), 1);
            let entry_params = &entries_params[0];
            assert_eq!(entry_params.amount, dec!(1000));
            // Credit balance should be offset with a debit
            assert_eq!(entry_params.direction, DebitOrCredit::Debit);
        }

        #[test]
        fn entry_params_generates_closing_entry_for_debit_balance() {
            let detail = create_detail_with_balance(dec!(500), DebitOrCredit::Debit);
            let entries_params = detail.entries_params();

            assert_eq!(entries_params.len(), 1);
            let entry_params = &entries_params[0];
            assert_eq!(entry_params.amount, dec!(500));
            // Debit balance should be offset with a credit
            assert_eq!(entry_params.direction, DebitOrCredit::Credit);
        }

        #[test]
        fn entry_params_empty_for_empty_detail() {
            let detail = ProfitAndLossLineItemDetail::from(HashMap::new());
            let entries_params = detail.entries_params();
            assert!(entries_params.is_empty());
        }

        #[test]
        fn iter_returns_all_balances() {
            let mut map = HashMap::new();

            let balance1 = create_balance_range(dec!(100), DebitOrCredit::Credit);
            let journal_id1 = cala_ledger::JournalId::new();
            let account_id1 = cala_ledger::AccountId::new();
            let currency: cala_ledger::Currency = "USD".parse().unwrap();
            map.insert((journal_id1, account_id1, currency), balance1);

            let balance2 = create_balance_range(dec!(200), DebitOrCredit::Debit);
            let journal_id2 = cala_ledger::JournalId::new();
            let account_id2 = cala_ledger::AccountId::new();
            map.insert((journal_id2, account_id2, currency), balance2);

            let detail = ProfitAndLossLineItemDetail::from(map);
            assert_eq!(detail.iter().count(), 2);
        }
    }

    mod adjusted_debit_or_credit {
        use super::*;
        use rust_decimal_macros::dec;

        #[test]
        fn positive_debit_balance_returns_credit() {
            let balance = create_balance_range(dec!(100), DebitOrCredit::Debit);
            let adjusted = AdjustedDebitOrCredit::new(&balance);
            assert_eq!(DebitOrCredit::from(adjusted), DebitOrCredit::Credit);
        }

        #[test]
        fn positive_credit_balance_returns_debit() {
            let balance = create_balance_range(dec!(100), DebitOrCredit::Credit);
            let adjusted = AdjustedDebitOrCredit::new(&balance);
            assert_eq!(DebitOrCredit::from(adjusted), DebitOrCredit::Debit);
        }

        #[test]
        fn zero_debit_balance_returns_credit() {
            let balance = create_balance_range(dec!(0), DebitOrCredit::Debit);
            let adjusted = AdjustedDebitOrCredit::new(&balance);
            // Zero is >= 0, so it flips
            assert_eq!(DebitOrCredit::from(adjusted), DebitOrCredit::Credit);
        }

        #[test]
        fn zero_credit_balance_returns_debit() {
            let balance = create_balance_range(dec!(0), DebitOrCredit::Credit);
            let adjusted = AdjustedDebitOrCredit::new(&balance);
            // Zero is >= 0, so it flips
            assert_eq!(DebitOrCredit::from(adjusted), DebitOrCredit::Debit);
        }

        #[test]
        fn negative_debit_balance_returns_debit() {
            // Contra-account: negative balance keeps same direction
            let balance = create_balance_range(dec!(-100), DebitOrCredit::Debit);
            let adjusted = AdjustedDebitOrCredit::new(&balance);
            assert_eq!(DebitOrCredit::from(adjusted), DebitOrCredit::Debit);
        }

        #[test]
        fn negative_credit_balance_returns_credit() {
            // Contra-account: negative balance keeps same direction
            let balance = create_balance_range(dec!(-100), DebitOrCredit::Credit);
            let adjusted = AdjustedDebitOrCredit::new(&balance);
            assert_eq!(DebitOrCredit::from(adjusted), DebitOrCredit::Credit);
        }
    }
}
