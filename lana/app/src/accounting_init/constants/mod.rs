mod credit_facilities;
mod deposits;

pub(super) use credit_facilities::*;
pub(super) use deposits::*;

pub(super) const LANA_JOURNAL_CODE: &str = "LANA_BANK_JOURNAL";

pub const CHART_REF: &str = "primary-chart";
pub(super) const CHART_NAME: &str = "Chart of Accounts";

pub const OBS_CHART_REF: &str = "off-balance-sheet-chart";
pub(super) const OBS_CHART_NAME: &str = "Off-Balance-Sheet Chart of Accounts";
