use rust_decimal::Decimal;

use crate::primitives::TransactionEntrySpec;

pub struct ProfitAndLossClosingDetails {
    pub net_category_balance: Decimal,
    pub closing_entries: Vec<TransactionEntrySpec>,
}
