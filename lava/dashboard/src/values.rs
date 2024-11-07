use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use lava_events::*;
use lava_money::{Satoshis, UsdCents};

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct DashboardValues {
    pub active_facilities: u32,
    pub pending_facilities: u32,
    pub total_disbursed: UsdCents,
    pub total_collateral: Satoshis,
    pub last_updated: DateTime<Utc>,
}

impl DashboardValues {
    pub(crate) fn process_event(&mut self, recorded_at: DateTime<Utc>, event: &LavaEvent) -> bool {
        self.last_updated = recorded_at;
        match event {
            LavaEvent::Credit(CreditEvent::FacilityCreated { .. }) => {
                self.pending_facilities += 1;
                true
            }
            LavaEvent::Credit(CreditEvent::FacilityActivated { .. }) => {
                self.pending_facilities -= 1;
                self.active_facilities += 1;
                true
            }
            LavaEvent::Credit(CreditEvent::FacilityCompleted { .. }) => {
                self.active_facilities -= 1;
                true
            }
            LavaEvent::Credit(CreditEvent::DisbursalSettled { amount, .. }) => {
                self.total_disbursed += *amount;
                true
            }
            LavaEvent::Credit(CreditEvent::FacilityRepaymentRecorded {
                disbursal_amount, ..
            }) => {
                self.total_disbursed -= *disbursal_amount;
                true
            }
            LavaEvent::Credit(CreditEvent::FacilityCollateralUpdated {
                abs_diff, action, ..
            }) => {
                match action {
                    FacilityCollateralUpdateAction::Add => {
                        self.total_collateral += *abs_diff;
                    }
                    FacilityCollateralUpdateAction::Remove => {
                        self.total_collateral -= *abs_diff;
                    }
                }
                true
            }
            _ => false,
        }
    }
}
