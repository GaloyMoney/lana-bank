use serde::{Deserialize, Serialize};

use crate::primitives::AccountCode;
use domain_config::{DomainConfigKey, DomainConfigValue};

pub const FISCAL_YEAR_CONFIG_KEY: &str = "fiscal-year-config";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalYearConfig {
    pub year_end_month: u32,
    pub revenue_code: AccountCode,
    pub cost_of_revenue_code: AccountCode,
    pub expenses_code: AccountCode,
    pub equity_retained_earnings_code: AccountCode,
    pub equity_retained_losses_code: AccountCode,
    // TODO: some type of frequency of closing type indication.
    // "monthly"/"quarterly" or "soft"/"hard".
    // closing_type_or_frequency: String,
}

impl DomainConfigValue for FiscalYearConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new(FISCAL_YEAR_CONFIG_KEY);
}
