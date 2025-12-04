use serde::{Deserialize, Serialize};

use crate::primitives::AccountCode;
use domain_config::{DomainConfigKey, DomainConfigValue};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalYearConfig {
    pub year_end_month: u32,
    pub revenue_code: AccountCode,
    pub cost_of_revenue_code: AccountCode,
    pub expenses_code: AccountCode,
    pub equity_retained_earnings_code: AccountCode,
    pub equity_retained_losses_code: AccountCode,
    // TODO: some type of frequency of closing type indication -
    // "monthly"/"quarterly" or "soft"/"hard" - especially if considering many jurisdictions
    // now.
    // closing_type_or_frequency: String,
}

impl DomainConfigValue for FiscalYearConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("fiscal-year-config");
}
