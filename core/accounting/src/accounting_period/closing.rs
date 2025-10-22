use rust_decimal::Decimal;

use crate::primitives::ClosingTxEntrySpec;

#[derive(Debug, Clone)]
pub struct ProfitAndLossClosingSpec {
    pub revenue: ProfitAndLossClosingCategory,
    pub cost_of_revenue: ProfitAndLossClosingCategory,
    pub expenses: ProfitAndLossClosingCategory,
}

impl ProfitAndLossClosingSpec {
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