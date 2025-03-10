mod credit_facilities;

pub(super) use credit_facilities::*;

pub(super) const LANA_JOURNAL_CODE: &str = "LANA_BANK_JOURNAL";

pub const CHART_REF: &str = "primary-chart";
pub(super) const CHART_NAME: &str = "Chart of Accounts";

pub const OBS_CHART_REF: &str = "off-balance-sheet-chart";
pub(super) const OBS_CHART_NAME: &str = "Off-Balance-Sheet Chart of Accounts";

pub const TRIAL_BALANCE_STATEMENT_NAME: &str = "Trial Balance";
pub const OBS_TRIAL_BALANCE_STATEMENT_NAME: &str = "Off-Balance-Sheet Trial Balance";

pub const PROFIT_AND_LOSS_STATEMENT_NAME: &str = "Profit & Loss Statement";

pub const CASH_FLOW_STATEMENT_NAME: &str = "Cash Flow Statement";

pub const BALANCE_SHEET_NAME: &str = "Balance Sheet";
pub const OBS_BALANCE_SHEET_NAME: &str = "Off-Balance-Sheet Balance Sheet";
