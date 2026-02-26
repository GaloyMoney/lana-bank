use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use money::{Satoshis, UsdCents};

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct DashboardValues {
    pub active_facilities: u32,
    pub pending_facilities: u32,
    pub total_disbursed: UsdCents,
    pub total_collateral: Satoshis,
    pub last_updated: DateTime<Utc>,
}
