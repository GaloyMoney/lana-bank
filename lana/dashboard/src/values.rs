use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};
use lana_events::*;

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct DashboardValues {
    pub active_facilities: u32,
    pub pending_facilities: u32,
    pub total_disbursed: UsdCents,
    pub total_collateral: Satoshis,
    pub last_updated: DateTime<Utc>,
}

impl DashboardValues {
    pub(crate) fn process_event(&mut self, recorded_at: DateTime<Utc>, event: &LanaEvent) -> bool {
        self.last_updated = recorded_at;
        match event {
            LanaEvent::Credit(CoreCreditEvent::FacilityProposalCreated { .. }) => {
                self.pending_facilities += 1;
                true
            }
            LanaEvent::Credit(CoreCreditEvent::FacilityActivated { .. }) => {
                self.pending_facilities -= 1;
                self.active_facilities += 1;
                true
            }
            LanaEvent::Credit(CoreCreditEvent::FacilityCompleted { .. }) => {
                self.active_facilities -= 1;
                true
            }
            LanaEvent::Credit(CoreCreditEvent::DisbursalSettled { amount, .. }) => {
                self.total_disbursed += *amount;
                true
            }
            LanaEvent::CreditCollection(CoreCreditCollectionEvent::PaymentAllocationCreated {
                entity,
            }) if entity.obligation_type == ObligationType::Disbursal => {
                self.total_disbursed -= entity.amount;
                true
            }
            LanaEvent::Credit(CoreCreditEvent::FacilityCollateralUpdated {
                abs_diff,
                direction: CollateralDirection::Add,
                ..
            }) => {
                self.total_collateral += *abs_diff;
                true
            }
            LanaEvent::Credit(CoreCreditEvent::FacilityCollateralUpdated {
                abs_diff,
                direction: CollateralDirection::Remove,
                ..
            }) => {
                self.total_collateral -= *abs_diff;
                true
            }
            _ => false,
        }
    }
}
