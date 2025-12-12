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

#[derive(Debug, Clone)]
pub(super) struct ClosingAccountBalance {
    pub(super) closed_settled_amount: Decimal,
    pub(super) normal_balance_type: DebitOrCredit,
}

impl From<&CalaBalanceRange> for ClosingAccountBalance {
    fn from(balance_range: &CalaBalanceRange) -> Self {
        Self {
            closed_settled_amount: balance_range.close.settled(),
            normal_balance_type: balance_range.close.balance_type,
        }
    }
}

pub struct OffsetDebitOrCredit(DebitOrCredit);

impl OffsetDebitOrCredit {
    fn new(b: &ClosingAccountBalance) -> Self {
        let amount = b.closed_settled_amount;
        let balance_type = b.normal_balance_type;

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

impl From<OffsetDebitOrCredit> for DebitOrCredit {
    fn from(offset_dir: OffsetDebitOrCredit) -> Self {
        offset_dir.0
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
        self.iter()
            .map(|(_, balance)| {
                let amount = balance.closed_settled_amount;
                match balance.normal_balance_type {
                    DebitOrCredit::Credit => amount,
                    DebitOrCredit::Debit => -amount,
                }
            })
            .sum()
    }

    fn entries_params(&self) -> Vec<EntryParams> {
        self.iter()
            .map(|((_, account_id, currency), balance)| {
                EntryParams::builder()
                    .account_id(*account_id)
                    .amount(balance.closed_settled_amount.abs())
                    .currency(*currency)
                    .direction(OffsetDebitOrCredit::new(balance).into())
                    .build()
                    .expect("Failed to build EntryParams")
            })
            .collect()
    }

    pub(super) fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, BalanceId, ClosingAccountBalance> {
        self.0.iter()
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
                    closed_settled_amount: revenue_amt,
                    normal_balance_type: DebitOrCredit::Credit,
                },
            );

            let mut cost_of_revenue_map = HashMap::new();
            cost_of_revenue_map.insert(
                cost_of_revenue_balance_id,
                ClosingAccountBalance {
                    closed_settled_amount: cost_of_revenue_amt,
                    normal_balance_type: DebitOrCredit::Debit,
                },
            );

            let mut expenses_map = HashMap::new();
            expenses_map.insert(
                expenses_balance_id,
                ClosingAccountBalance {
                    closed_settled_amount: expenses_amt,
                    normal_balance_type: DebitOrCredit::Debit,
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
