use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use lava_events::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditFacilityReceivable {
    pub disbursed: u64,
    pub interest: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerSummaryValues {
    pub id: uuid::Uuid,
    pub receivable: CreditFacilityReceivable,
    pub last_updated: DateTime<Utc>,
}

impl CustomerSummaryValues {
    pub(crate) fn process_event(&mut self, recorded_at: DateTime<Utc>, event: &LavaEvent) -> bool {
        self.last_updated = recorded_at;
        match event {
            LavaEvent::Credit(CreditEvent::CreditFacilityFundsDisbursed { amount, .. }) => {
                self.receivable.disbursed += amount;
                true
            }
            _ => false,
        }
    }
}
