use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::primitives::CalaTxId;

use super::template::EntryParams;

use cala_ledger::{
    BalanceId, Currency as CalaCurrency, DebitOrCredit, account::Account,
    account_set::AccountSetId, balance::BalanceRange as CalaBalanceRange,
};

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

impl ClosingTxParams {
    pub(super) fn retained_earnings_account_name(&self) -> String {
        format!("NET_INCOME_{}", self.description)
    }
}

#[derive(Debug, Clone)]
pub(super) struct ClosingAccountBalance {
    pub(super) amount: Decimal,
    pub(super) direction: DebitOrCredit,
}

impl From<&CalaBalanceRange> for ClosingAccountBalance {
    fn from(balance_range: &CalaBalanceRange) -> Self {
        Self {
            amount: balance_range.close.settled(),
            direction: balance_range.close.balance_type,
        }
    }
}

impl ClosingAccountBalance {
    fn abs(&self) -> Decimal {
        self.amount.abs()
    }

    fn direction_for_offsetting_entry(&self) -> DebitOrCredit {
        if self.amount.is_sign_negative() {
            self.direction
        } else {
            match self.direction {
                DebitOrCredit::Debit => DebitOrCredit::Credit,
                DebitOrCredit::Credit => DebitOrCredit::Debit,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ProfitAndLossLineItemDetail(HashMap<BalanceId, ClosingAccountBalance>);

impl From<HashMap<BalanceId, CalaBalanceRange>> for ProfitAndLossLineItemDetail {
    fn from(account_balances: HashMap<BalanceId, CalaBalanceRange>) -> Self {
        Self(
            account_balances
                .into_iter()
                .map(|(k, v)| (k, ClosingAccountBalance::from(&v)))
                .collect(),
        )
    }
}

impl ProfitAndLossLineItemDetail {
    fn contributions(&self) -> Decimal {
        self.0
            .values()
            .map(|balance| {
                let amount = balance.amount;
                match balance.direction {
                    DebitOrCredit::Credit => amount,
                    DebitOrCredit::Debit => -amount,
                }
            })
            .sum()
    }

    fn entries_params(&self) -> Vec<EntryParams> {
        self.0
            .iter()
            .filter(|(_, balance)| !balance.abs().is_zero())
            .map(|((_, account_id, currency), balance)| {
                EntryParams::builder()
                    .account_id(*account_id)
                    .amount(balance.abs())
                    .currency(*currency)
                    .direction(balance.direction_for_offsetting_entry())
                    .build()
                    .expect("Failed to build EntryParams")
            })
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
    fn contributions(&self) -> Decimal {
        self.revenue.contributions()
            + self.cost_of_revenue.contributions()
            + self.expenses.contributions()
    }

    pub(super) fn entries_params(&self, retained_earnings_account: Account) -> Vec<EntryParams> {
        let retained_earnings_entry = vec![
            EntryParams::builder()
                .account_id(retained_earnings_account.id)
                .amount(self.contributions().abs())
                .currency(CalaCurrency::USD)
                .direction(retained_earnings_account.values().normal_balance_type)
                .build()
                .expect("Failed to build EntryParams"),
        ];

        self.revenue
            .entries_params()
            .into_iter()
            .chain(self.cost_of_revenue.entries_params())
            .chain(self.expenses.entries_params())
            .chain(retained_earnings_entry)
            .collect()
    }

    pub(super) fn retained_earnings(
        &self,
        retained_earnings_gain_account_id: AccountSetId,
        retained_earnings_loss_account_id: AccountSetId,
    ) -> (DebitOrCredit, AccountSetId) {
        if self.contributions() >= Decimal::ZERO {
            (DebitOrCredit::Credit, retained_earnings_gain_account_id)
        } else {
            (DebitOrCredit::Debit, retained_earnings_loss_account_id)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod closing_account_balance {
        use rust_decimal_macros::dec;

        use super::*;

        fn balance(amount: Decimal, direction: DebitOrCredit) -> ClosingAccountBalance {
            ClosingAccountBalance { amount, direction }
        }

        #[test]
        fn negative_balance_offsets_with_same_side_entry() {
            let bal = balance(dec!(-100), DebitOrCredit::Credit);
            assert_eq!(bal.direction_for_offsetting_entry(), DebitOrCredit::Credit);

            let bal = balance(dec!(-100), DebitOrCredit::Debit);
            assert_eq!(bal.direction_for_offsetting_entry(), DebitOrCredit::Debit);
        }

        #[test]
        fn positive_credit_balance_offsets_with_opposite_side_entry() {
            let bal = balance(dec!(100), DebitOrCredit::Credit);
            assert_eq!(bal.direction_for_offsetting_entry(), DebitOrCredit::Debit);

            let bal = balance(dec!(100), DebitOrCredit::Debit);
            assert_eq!(bal.direction_for_offsetting_entry(), DebitOrCredit::Credit);
        }

        #[test]
        fn zero_credit_balance_offsets_with_opposite_side_entry() {
            let bal = balance(dec!(0), DebitOrCredit::Credit);
            assert_eq!(bal.direction_for_offsetting_entry(), DebitOrCredit::Debit);

            let bal = balance(dec!(0), DebitOrCredit::Debit);
            assert_eq!(bal.direction_for_offsetting_entry(), DebitOrCredit::Credit);
        }
    }

    mod profit_and_loss_line_item_detail {
        use cala_ledger::{AccountId, DebitOrCredit, JournalId};
        use rust_decimal::Decimal;
        use rust_decimal_macros::dec;

        use super::*;

        mod entries_params {
            use super::*;

            fn line_item_with_zero_balance_account() -> ProfitAndLossLineItemDetail {
                let journal_id = JournalId::new();
                let mut balances = HashMap::new();
                balances.insert(
                    (journal_id, AccountId::new(), CalaCurrency::USD),
                    ClosingAccountBalance {
                        amount: dec!(0),
                        direction: DebitOrCredit::Credit,
                    },
                );
                balances.insert(
                    (journal_id, AccountId::new(), CalaCurrency::USD),
                    ClosingAccountBalance {
                        amount: dec!(100),
                        direction: DebitOrCredit::Credit,
                    },
                );
                ProfitAndLossLineItemDetail(balances)
            }

            #[test]
            fn skips_entry_param_for_zero_balance_account() {
                let line_item = line_item_with_zero_balance_account();
                let entries = line_item.entries_params();
                assert_eq!(entries.len(), 1);
            }
        }

        mod contributions {
            use super::*;

            fn line_item_with(
                balances: Vec<(Decimal, DebitOrCredit)>,
            ) -> ProfitAndLossLineItemDetail {
                let journal_id = JournalId::new();
                let mut map = HashMap::new();
                for (amount, direction) in balances {
                    map.insert(
                        (journal_id, AccountId::new(), CalaCurrency::USD),
                        ClosingAccountBalance { amount, direction },
                    );
                }
                ProfitAndLossLineItemDetail(map)
            }

            #[test]
            fn empty_line_item_returns_zero() {
                let line_item = ProfitAndLossLineItemDetail(HashMap::new());
                assert_eq!(line_item.contributions(), Decimal::ZERO);
            }

            #[test]
            fn single_credit_balance_returns_positive() {
                let line_item = line_item_with(vec![(dec!(100), DebitOrCredit::Credit)]);
                assert_eq!(line_item.contributions(), dec!(100));
            }

            #[test]
            fn single_debit_balance_returns_negative() {
                let line_item = line_item_with(vec![(dec!(100), DebitOrCredit::Debit)]);
                assert_eq!(line_item.contributions(), dec!(-100));
            }

            #[test]
            fn negative_credit_balance_returns_negative() {
                let line_item = line_item_with(vec![(dec!(-100), DebitOrCredit::Credit)]);
                assert_eq!(line_item.contributions(), dec!(-100));
            }

            #[test]
            fn negative_debit_balance_returns_positive() {
                let line_item = line_item_with(vec![(dec!(-100), DebitOrCredit::Debit)]);
                assert_eq!(line_item.contributions(), dec!(100));
            }
        }
    }

    mod closing_profit_and_loss_account_balances {
        use super::*;

        mod retained_earnings {
            use cala_ledger::{AccountId, JournalId, account_set::AccountSetId};
            use rust_decimal_macros::dec;

            use super::*;

            fn gain_account_set_id() -> AccountSetId {
                AccountSetId::new()
            }

            fn loss_account_set_id() -> AccountSetId {
                AccountSetId::new()
            }

            fn empty_balances() -> ClosingProfitAndLossAccountBalances {
                ClosingProfitAndLossAccountBalances {
                    revenue: ProfitAndLossLineItemDetail(HashMap::new()),
                    cost_of_revenue: ProfitAndLossLineItemDetail(HashMap::new()),
                    expenses: ProfitAndLossLineItemDetail(HashMap::new()),
                }
            }

            fn balances_with(
                revenue_amt: Decimal,
                cost_of_revenue_amt: Decimal,
                expenses_amt: Decimal,
            ) -> ClosingProfitAndLossAccountBalances {
                let journal_id = JournalId::new();

                let revenue_balance_id = (journal_id, AccountId::new(), CalaCurrency::USD);
                let cost_of_revenue_balance_id = (journal_id, AccountId::new(), CalaCurrency::USD);
                let expenses_balance_id = (journal_id, AccountId::new(), CalaCurrency::USD);

                let mut revenue_map = HashMap::new();
                revenue_map.insert(
                    revenue_balance_id,
                    ClosingAccountBalance {
                        amount: revenue_amt,
                        direction: DebitOrCredit::Credit,
                    },
                );

                let mut cost_of_revenue_map = HashMap::new();
                cost_of_revenue_map.insert(
                    cost_of_revenue_balance_id,
                    ClosingAccountBalance {
                        amount: cost_of_revenue_amt,
                        direction: DebitOrCredit::Debit,
                    },
                );

                let mut expenses_map = HashMap::new();
                expenses_map.insert(
                    expenses_balance_id,
                    ClosingAccountBalance {
                        amount: expenses_amt,
                        direction: DebitOrCredit::Debit,
                    },
                );

                ClosingProfitAndLossAccountBalances {
                    revenue: ProfitAndLossLineItemDetail(revenue_map),
                    cost_of_revenue: ProfitAndLossLineItemDetail(cost_of_revenue_map),
                    expenses: ProfitAndLossLineItemDetail(expenses_map),
                }
            }

            #[test]
            fn returns_gain_account_with_credit_for_zero_contributions() {
                let balances = empty_balances();
                let gain_id = gain_account_set_id();
                let loss_id = loss_account_set_id();

                let (direction, account_id) = balances.retained_earnings(gain_id, loss_id);

                assert_eq!(direction, DebitOrCredit::Credit);
                assert_eq!(account_id, gain_id);
            }

            #[test]
            fn returns_gain_account_with_credit_for_positive_contributions() {
                // Revenue > (cost_of_revenue + expenses) => positive net income
                let balances = balances_with(dec!(1000), dec!(300), dec!(200));
                let gain_id = gain_account_set_id();
                let loss_id = loss_account_set_id();

                let (direction, account_id) = balances.retained_earnings(gain_id, loss_id);

                assert_eq!(direction, DebitOrCredit::Credit);
                assert_eq!(account_id, gain_id);
            }

            #[test]
            fn returns_loss_account_with_debit_for_negative_contributions() {
                // Revenue < (cost_of_revenue + expenses) => negative net income (loss)
                let balances = balances_with(dec!(100), dec!(300), dec!(200));
                let gain_id = gain_account_set_id();
                let loss_id = loss_account_set_id();

                let (direction, account_id) = balances.retained_earnings(gain_id, loss_id);

                assert_eq!(direction, DebitOrCredit::Debit);
                assert_eq!(account_id, loss_id);
            }
        }
    }
}
