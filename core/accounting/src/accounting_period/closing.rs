use std::collections::HashMap;

use cala_ledger::{
    AccountId as CalaAccountId, Currency as CalaCurrency, DebitOrCredit,
    JournalId as CalaJournalId, balance::BalanceRange as CalaBalanceRange,
};
use rust_decimal::Decimal;

use crate::LedgerAccountId;

#[derive(Debug, Clone)]
pub struct ProfitAndLossClosingSpec {
    revenue: ProfitAndLossClosingCategory,
    cost_of_revenue: ProfitAndLossClosingCategory,
    expenses: ProfitAndLossClosingCategory,
}

impl ProfitAndLossClosingSpec {
    pub fn new(
        revenue: ProfitAndLossClosingCategory,
        cost_of_revenue: ProfitAndLossClosingCategory,
        expenses: ProfitAndLossClosingCategory,
    ) -> Self {
        Self {
            revenue,
            cost_of_revenue,
            expenses,
        }
    }
    pub(super) fn net_income(&self) -> Decimal {
        self.revenue.net_category_balance
            - self.cost_of_revenue.net_category_balance
            - self.expenses.net_category_balance
    }

    pub(super) fn take_profit_and_loss_entries(&mut self) -> Vec<ClosingTxEntrySpec> {
        let mut tx_entries = Vec::new();
        tx_entries.append(&mut self.revenue.closing_entries);
        tx_entries.append(&mut self.expenses.closing_entries);
        tx_entries.append(&mut self.cost_of_revenue.closing_entries);

        tx_entries
    }
}

#[derive(Debug, Clone)]
pub struct ProfitAndLossClosingCategory {
    pub net_category_balance: Decimal,
    pub closing_entries: Vec<ClosingTxEntrySpec>,
}

#[derive(Debug, Clone)]
pub struct ClosingTxEntrySpec {
    pub account_id: LedgerAccountId,
    pub amount: Decimal,
    pub currency: CalaCurrency,
    pub description: String,
    pub direction: DebitOrCredit,
}

impl ClosingTxEntrySpec {
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
    pub revenue: HashMap<(CalaJournalId, CalaAccountId, CalaCurrency), CalaBalanceRange>,
    pub cost_of_revenue: HashMap<(CalaJournalId, CalaAccountId, CalaCurrency), CalaBalanceRange>,
    pub expenses: HashMap<(CalaJournalId, CalaAccountId, CalaCurrency), CalaBalanceRange>,
}
