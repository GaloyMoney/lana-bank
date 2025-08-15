use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_customer::CustomerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerActivity {
    pub customer_id: CustomerId,
    pub last_activity_date: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CustomerActivity {
    pub fn new(customer_id: CustomerId, activity_date: DateTime<Utc>) -> Self {
        Self {
            customer_id,
            last_activity_date: activity_date,
            updated_at: Utc::now(),
        }
    }

    pub fn update_activity(&mut self, activity_date: DateTime<Utc>) {
        if activity_date > self.last_activity_date {
            self.last_activity_date = activity_date;
            self.updated_at = Utc::now();
        }
    }
}
