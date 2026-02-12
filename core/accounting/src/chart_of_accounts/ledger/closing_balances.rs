use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::primitives::{ClosingAccountSetIds, ClosingTxDetails};

use super::templates::EntryParams;

use cala_ledger::{
    AccountId, BalanceId, Currency as CalaCurrency, DebitOrCredit,
    account::{Account, NewAccount},
    account_set::AccountSetId,
    balance::BalanceRange as CalaBalanceRange,
};

#[derive(Debug)]
pub(crate) struct NetIncomeAccountSetIds {
    pub(crate) revenue: AccountSetId,
    pub(crate) cost_of_revenue: AccountSetId,
    pub(crate) expenses: AccountSetId,
}

#[derive(Debug)]
pub(crate) struct RetainedEarningsAccountSetIds {
    pub(crate) equity_retained_earnings: AccountSetId,
    pub(crate) equity_retained_losses: AccountSetId,
}

#[derive(Debug)]
pub(crate) struct ClosingTxParentIdsAndDetails {
    pub(crate) tx_details: ClosingTxDetails,
    pub(crate) net_income_parent_ids: NetIncomeAccountSetIds,
    pub(crate) retained_earnings_parent_ids: RetainedEarningsAccountSetIds,
}

impl ClosingTxParentIdsAndDetails {
    pub(crate) fn new(account_set_ids: ClosingAccountSetIds, tx_details: ClosingTxDetails) -> Self {
        Self {
            tx_details,

            net_income_parent_ids: NetIncomeAccountSetIds {
                revenue: account_set_ids.revenue,
                cost_of_revenue: account_set_ids.cost_of_revenue,
                expenses: account_set_ids.expenses,
            },

            retained_earnings_parent_ids: RetainedEarningsAccountSetIds {
                equity_retained_earnings: account_set_ids.equity_retained_earnings,
                equity_retained_losses: account_set_ids.equity_retained_losses,
            },
        }
    }

    pub(crate) fn posted_as_of(&self) -> NaiveDate {
        self.tx_details.effective_balances_until
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
pub(super) struct NewAccountDetails {
    pub(super) new_account: NewAccount,
    pub(super) parent_account_set_id: AccountSetId,
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

    fn net_income_entries(&self) -> Vec<EntryParams> {
        self.revenue
            .entries_params()
            .into_iter()
            .chain(self.cost_of_revenue.entries_params())
            .chain(self.expenses.entries_params())
            .collect::<Vec<EntryParams>>()
    }

    fn has_retained_earnings_entry(&self) -> bool {
        !self.net_income_entries().is_empty()
    }

    fn retained_earnings_direction(
        &self,
        RetainedEarningsAccountSetIds {
            equity_retained_earnings: gain_account_id,
            equity_retained_losses: loss_account_id,
        }: RetainedEarningsAccountSetIds,
    ) -> (DebitOrCredit, AccountSetId) {
        if self.contributions() >= Decimal::ZERO {
            (DebitOrCredit::Credit, gain_account_id)
        } else {
            (DebitOrCredit::Debit, loss_account_id)
        }
    }

    pub(super) fn retained_earnings_new_account(
        &self,
        name: String,
        parent_account_set_ids: RetainedEarningsAccountSetIds,
    ) -> Option<NewAccountDetails> {
        if !self.has_retained_earnings_entry() {
            return None;
        }

        let (normal_balance_type, parent_account_set_id) =
            self.retained_earnings_direction(parent_account_set_ids);

        let id = AccountId::new();
        let new_account = NewAccount::builder()
            .id(id)
            .name(name)
            .code(id.to_string())
            .normal_balance_type(normal_balance_type)
            .build()
            .expect("Could not build new account for annual close net income transfer entry");

        Some(NewAccountDetails {
            new_account,
            parent_account_set_id,
        })
    }

    pub(super) fn entries_params(
        &self,
        retained_earnings_account: Option<Account>,
    ) -> Vec<EntryParams> {
        let mut entries = vec![];
        if let Some(retained_earnings_account) = retained_earnings_account {
            entries.extend(self.net_income_entries());

            let retained_earnings_entry = EntryParams::builder()
                .account_id(retained_earnings_account.id)
                .amount(self.contributions().abs())
                .currency(CalaCurrency::USD)
                .direction(retained_earnings_account.values().normal_balance_type)
                .build()
                .expect("Failed to build EntryParams");
            entries.push(retained_earnings_entry);
        }

        entries
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
        use cala_ledger::{AccountId, JournalId, TransactionId};

        use super::*;

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

        fn empty_balances() -> ClosingProfitAndLossAccountBalances {
            ClosingProfitAndLossAccountBalances {
                revenue: ProfitAndLossLineItemDetail(HashMap::new()),
                cost_of_revenue: ProfitAndLossLineItemDetail(HashMap::new()),
                expenses: ProfitAndLossLineItemDetail(HashMap::new()),
            }
        }

        fn dummy_params() -> ClosingTxParentIdsAndDetails {
            ClosingTxParentIdsAndDetails {
                tx_details: ClosingTxDetails {
                    tx_id: TransactionId::new(),
                    description: "".to_string(),
                    effective_balances_from: chrono::Utc::now().date_naive(),
                    effective_balances_until: chrono::Utc::now().date_naive(),
                },
                net_income_parent_ids: NetIncomeAccountSetIds {
                    revenue: AccountSetId::new(),
                    cost_of_revenue: AccountSetId::new(),
                    expenses: AccountSetId::new(),
                },
                retained_earnings_parent_ids: RetainedEarningsAccountSetIds {
                    equity_retained_earnings: AccountSetId::new(),
                    equity_retained_losses: AccountSetId::new(),
                },
            }
        }

        mod entries_params {
            use cala_ledger::account::NewAccount;
            use es_entity::{IntoEvents, TryFromEvents};
            use rust_decimal_macros::dec;

            use crate::LedgerAccountId;

            use super::*;

            fn random_account() -> Account {
                let account_id = LedgerAccountId::new();
                let new_account = NewAccount::builder()
                    .name(account_id.to_string())
                    .id(account_id)
                    .code(account_id.to_string())
                    .build()
                    .unwrap();
                Account::try_from_events(new_account.into_events()).unwrap()
            }

            #[test]
            fn returns_empty_vec_for_empty_balances() {
                let balances = empty_balances();

                let retained_earnings_account = balances.retained_earnings_new_account(
                    "".to_string(),
                    dummy_params().retained_earnings_parent_ids,
                );
                assert!(retained_earnings_account.is_none());

                let entries = balances.entries_params(None);
                assert!(entries.is_empty());
            }

            #[test]
            fn returns_entries_for_each_account_plus_retained_earnings() {
                // 3 accounts with non-zero balances + 1 retained earnings = 4 entries
                let balances = balances_with(dec!(1000), dec!(300), dec!(100));
                let retained_earnings_account = random_account();

                let entries = balances.entries_params(Some(retained_earnings_account));

                assert_eq!(entries.len(), 4);
            }

            #[test]
            fn returned_entries_sum_to_zero() {
                let balances = balances_with(dec!(1000), dec!(300), dec!(100));
                let retained_earnings_account = random_account();

                let entries = balances.entries_params(Some(retained_earnings_account));

                let sum: Decimal = entries
                    .iter()
                    .map(|e| match e.direction {
                        DebitOrCredit::Debit => e.amount,
                        DebitOrCredit::Credit => -e.amount,
                    })
                    .sum();

                assert_eq!(sum, Decimal::ZERO);
            }

            #[test]
            fn retained_earnings_entry_has_absolute_contribution_amount() {
                // Net income = 1000 (credit) - 300 (debit) - 100 (debit) = 600
                let balances = balances_with(dec!(1000), dec!(300), dec!(100));
                let retained_earnings_account = random_account();

                let entries = balances.entries_params(Some(retained_earnings_account));
                let retained_earnings_entry = entries.last().unwrap();

                assert_eq!(retained_earnings_entry.amount, dec!(600));
            }
        }

        mod retained_earnings_direction {
            use rust_decimal_macros::dec;

            use super::*;

            #[test]
            fn returns_gain_account_with_credit_for_zero_contributions() {
                let balances = empty_balances();
                let ClosingTxParentIdsAndDetails {
                    retained_earnings_parent_ids:
                        parent_ids @ RetainedEarningsAccountSetIds {
                            equity_retained_earnings: gain_id,
                            ..
                        },
                    ..
                } = dummy_params();

                let (direction, account_id) = balances.retained_earnings_direction(parent_ids);

                assert_eq!(direction, DebitOrCredit::Credit);
                assert_eq!(account_id, gain_id);
            }

            #[test]
            fn returns_gain_account_with_credit_for_positive_contributions() {
                // Revenue > (cost_of_revenue + expenses) => positive net income
                let balances = balances_with(dec!(1000), dec!(300), dec!(200));
                let ClosingTxParentIdsAndDetails {
                    retained_earnings_parent_ids:
                        parent_ids @ RetainedEarningsAccountSetIds {
                            equity_retained_earnings: gain_id,
                            ..
                        },
                    ..
                } = dummy_params();

                let (direction, account_id) = balances.retained_earnings_direction(parent_ids);

                assert_eq!(direction, DebitOrCredit::Credit);
                assert_eq!(account_id, gain_id);
            }

            #[test]
            fn returns_loss_account_with_debit_for_negative_contributions() {
                // Revenue < (cost_of_revenue + expenses) => negative net income (loss)
                let balances = balances_with(dec!(100), dec!(300), dec!(200));
                let ClosingTxParentIdsAndDetails {
                    retained_earnings_parent_ids:
                        parent_ids @ RetainedEarningsAccountSetIds {
                            equity_retained_losses: loss_id,
                            ..
                        },
                    ..
                } = dummy_params();

                let (direction, account_id) = balances.retained_earnings_direction(parent_ids);

                assert_eq!(direction, DebitOrCredit::Debit);
                assert_eq!(account_id, loss_id);
            }
        }
    }
}
