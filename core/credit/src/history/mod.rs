mod entry;
pub mod error;
mod repo;

use crate::event::CoreCreditEvent;
pub use entry::*;
pub use repo::HistoryRepo;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CreditFacilityHistory {
    pub(super) entries: Vec<CreditFacilityHistoryEntry>,
}

impl CreditFacilityHistory {
    pub fn process_event(&mut self, event: &CoreCreditEvent) {
        use CoreCreditEvent::*;

        match event {
            FacilityCreated { .. } => {}
            FacilityApproved { .. } => {}
            FacilityActivated {
                activation_tx_id,
                activated_at,
                amount,
                ..
            } => {
                self.entries.push(CreditFacilityHistoryEntry::Origination(
                    CreditFacilityOrigination {
                        cents: *amount,
                        recorded_at: *activated_at,
                        tx_id: *activation_tx_id,
                    },
                ));
            }
            FacilityCollateralUpdated {
                abs_diff,
                recorded_at,
                action,
                ledger_tx_id,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: *abs_diff,
                        recorded_at: *recorded_at,
                        action: *action,
                        tx_id: *ledger_tx_id,
                    }));
            }
            FacilityCollateralizationChanged {
                state,
                recorded_at,
                outstanding,
                price,
                collateral,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateralization(
                        CollateralizationUpdated {
                            state: *state,
                            collateral: *collateral,
                            outstanding_interest: outstanding.interest,
                            outstanding_disbursal: outstanding.disbursed,
                            recorded_at: *recorded_at,
                            price: *price,
                        },
                    ));
            }
            FacilityRepaymentRecorded {
                payment_id,
                amount,
                recorded_at,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Payment(IncrementalPayment {
                        recorded_at: *recorded_at,
                        cents: *amount,
                        payment_id: *payment_id,
                    }));
            }
            DisbursalSettled {
                amount,
                recorded_at,
                ledger_tx_id,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Disbursal(DisbursalExecuted {
                        cents: *amount,
                        recorded_at: *recorded_at,
                        tx_id: *ledger_tx_id,
                    }));
            }
            AccrualPosted {
                amount,
                period,
                ledger_tx_id,
                ..
            } => {
                self.entries.push(CreditFacilityHistoryEntry::Interest(
                    InterestAccrualsPosted {
                        cents: *amount,
                        recorded_at: period.end, // change when we have effective
                        tx_id: *ledger_tx_id,
                        days: period.days(),
                    },
                ));
            }
            FacilityCompleted { .. } => {}
            ObligationCreated { .. } => {}
            ObligationDue { .. } => {}
            ObligationOverdue { .. } => {}
            ObligationDefaulted { .. } => {}
            ObligationCompleted { .. } => {}
        }
    }
}
