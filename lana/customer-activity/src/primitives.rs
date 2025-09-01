use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_customer::CustomerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerActivity {
    pub customer_id: CustomerId,
    pub last_activity_date: DateTime<Utc>,
}
