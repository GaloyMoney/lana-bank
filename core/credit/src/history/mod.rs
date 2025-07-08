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
                self.entries.push(CreditFacilityHistoryEntry::Approved(
                    CreditFacilityApproved {
                        cents: *amount,
                        recorded_at: *activated_at,
                        effective: activated_at.date_naive(),
                        tx_id: *activation_tx_id,
                    },
                ));
            }
            FacilityCollateralUpdated {
                abs_diff,
                recorded_at,
                effective,
                action,
                ledger_tx_id,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: *abs_diff,
                        recorded_at: *recorded_at,
                        effective: *effective,
                        action: *action,
                        tx_id: *ledger_tx_id,
                    }));
            }
            FacilityCollateralizationChanged {
                state,
                recorded_at,
                effective,
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
                            effective: *effective,
                            price: *price,
                        },
                    ));
            }
            FacilityRepaymentRecorded {
                payment_id,
                amount,
                recorded_at,
                effective,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Payment(IncrementalPayment {
                        recorded_at: *recorded_at,
                        effective: *effective,
                        cents: *amount,
                        payment_id: *payment_id,
                    }));
            }
            DisbursalSettled {
                amount,
                recorded_at,
                effective,
                ledger_tx_id,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Disbursal(DisbursalExecuted {
                        cents: *amount,
                        recorded_at: *recorded_at,
                        effective: *effective,
                        tx_id: *ledger_tx_id,
                    }));
            }
            AccrualPosted {
                amount,
                period,
                ledger_tx_id,
                recorded_at,
                effective,
                ..
            } => {
                self.entries.push(CreditFacilityHistoryEntry::Interest(
                    InterestAccrualsPosted {
                        cents: *amount,
                        recorded_at: *recorded_at,
                        effective: *effective,
                        tx_id: *ledger_tx_id,
                        days: period.days(),
                    },
                ));
            }
            LiquidationProcessStarted {
                amount,
                recorded_at,
                effective,
                ledger_tx_id,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::ReservedForLiquidation(
                        ObligationMovedToLiquidation {
                            cents: *amount,
                            recorded_at: *recorded_at,
                            effective: *effective,
                            tx_id: *ledger_tx_id,
                        },
                    ));
            }
            FacilityCompleted { .. } => {}
            ObligationCreated { .. } => {}
            ObligationDue { .. } => {}
            ObligationOverdue { .. } => {}
            ObligationDefaulted { .. } => {}
            LiquidationProcessConcluded { .. } => {}
            ObligationCompleted { .. } => {}
        }
    }
}
