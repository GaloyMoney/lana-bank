use std::collections::HashMap;

use cala_ledger::{
    BalanceId, Currency as CalaCurrency, DebitOrCredit, balance::BalanceRange as CalaBalanceRange,
};
use rust_decimal::Decimal;

use crate::LedgerAccountId;

#[derive(Debug, Clone)]
pub struct AccountClosingEntry {
    pub account_id: LedgerAccountId,
    pub amount: Decimal,
    pub currency: CalaCurrency,
    pub description: String,
    pub direction: DebitOrCredit,
}

impl AccountClosingEntry {
    pub fn new(
        account_id: LedgerAccountId,
        amount: Decimal,
        currency: CalaCurrency,
        description: String,
        direction: DebitOrCredit,
    ) -> Self {
        Self {
            account_id,
            amount,
            currency,
            description,
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
    pub fn into_closing_entries(&self) -> (Decimal, Vec<AccountClosingEntry>) {
        let (revenue_balance, mut revenue) = Self::create_account_closing_summary(&self.revenue);
        let (cost_of_revenue_balance, mut cost_of_revenue) =
            Self::create_account_closing_summary(&self.cost_of_revenue);
        let (expenses_balance, mut expenses) = Self::create_account_closing_summary(&self.expenses);

        let net_income = revenue_balance - cost_of_revenue_balance - expenses_balance;

        let mut entries = Vec::new();
        entries.append(&mut revenue);
        entries.append(&mut expenses);
        entries.append(&mut cost_of_revenue);

        (net_income, entries)
    }

    fn create_account_closing_summary(
        balances: &HashMap<BalanceId, CalaBalanceRange>,
    ) -> (Decimal, Vec<AccountClosingEntry>) {
        let mut net_balance: Decimal = Decimal::from(0);
        let mut closing_entries = Vec::new();
        for ((_, account_id, currency), balance) in balances {
            let amount = balance.close.settled().abs();
            net_balance += amount;
            let direction = if balance.close.balance_type == DebitOrCredit::Debit {
                DebitOrCredit::Credit
            } else {
                DebitOrCredit::Debit
            };

            closing_entries.push(AccountClosingEntry::new(
                (*account_id).into(),
                amount,
                *currency,
                "Annual Close Offset".to_string(),
                direction,
            ));
        }

        (net_balance, closing_entries)
    }
}
