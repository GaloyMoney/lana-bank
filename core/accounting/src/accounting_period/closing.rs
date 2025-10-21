use std::collections::HashMap;

use cala_ledger::{
    BalanceId, Currency as CalaCurrency, DebitOrCredit, balance::BalanceRange as CalaBalanceRange,
};
use rust_decimal::Decimal;

use crate::LedgerAccountId;

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
    /// Creates closing entries for the `ProfitAndLossStatement`
    /// underlying accounts that is valid at any time during the
    /// closing grace period. Notably, this does not create the equity
    /// closing entry.
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
